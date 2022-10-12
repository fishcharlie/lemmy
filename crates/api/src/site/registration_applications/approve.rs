use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{ApproveRegistrationApplication, RegistrationApplicationResponse},
  utils::{
    blocking,
    get_local_user_view_from_jwt,
    is_admin,
    local_site_to_email_config,
    send_application_approved_email,
  },
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    local_user::{LocalUser, LocalUserUpdateForm},
    registration_application::{RegistrationApplication, RegistrationApplicationUpdateForm},
  },
  traits::Crud,
  utils::diesel_option_overwrite,
};
use lemmy_db_views::structs::{LocalUserView, RegistrationApplicationView};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for ApproveRegistrationApplication {
  type Response = RegistrationApplicationResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let app_id = data.id;

    // Only let admins do this
    is_admin(&local_user_view)?;

    // Update the registration with reason, admin_id
    let deny_reason = diesel_option_overwrite(&data.deny_reason);
    let app_form = RegistrationApplicationUpdateForm {
      admin_id: Some(Some(local_user_view.person.id)),
      deny_reason,
    };

    let registration_application = blocking(context.pool(), move |conn| {
      RegistrationApplication::update(conn, app_id, &app_form)
    })
    .await??;

    // Update the local_user row
    let local_user_form = LocalUserUpdateForm::builder()
      .accepted_application(Some(data.approve))
      .build();

    let approved_user_id = registration_application.local_user_id;
    blocking(context.pool(), move |conn| {
      LocalUser::update(conn, approved_user_id, &local_user_form)
    })
    .await??;

    if data.approve {
      let approved_local_user_view = blocking(context.pool(), move |conn| {
        LocalUserView::read(conn, approved_user_id)
      })
      .await??;

      if approved_local_user_view.local_user.email.is_some() {
        let local_site = blocking(context.pool(), LocalSite::read).await??;
        let email_config = local_site_to_email_config(&local_site)?;
        send_application_approved_email(
          &approved_local_user_view,
          context.settings(),
          &email_config,
        )?;
      }
    }

    // Read the view
    let registration_application = blocking(context.pool(), move |conn| {
      RegistrationApplicationView::read(conn, app_id)
    })
    .await??;

    Ok(Self::Response {
      registration_application,
    })
  }
}
