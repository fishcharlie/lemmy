use crate::{aggregates::structs::SiteAggregates, schema::site_aggregates};
use diesel::{result::Error, *};

impl SiteAggregates {
  pub fn read(conn: &mut PgConnection) -> Result<Self, Error> {
    site_aggregates::table.first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::site_aggregates::SiteAggregates,
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::{Instance, InstanceForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
      site::{Site, SiteInsertForm},
    },
    traits::Crud,
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let new_instance = InstanceForm {
      domain: "my_domain.tld".into(),
      updated: None,
    };

    let inserted_instance = Instance::create(conn, &new_instance).unwrap();

    let new_person = PersonInsertForm::builder()
      .name("thommy_site_agg".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(conn, &new_person).unwrap();

    let site_form = SiteInsertForm::builder()
      .name("test_site".into())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_site = Site::create(conn, &site_form).unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("TIL_site_agg".into())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(conn, &new_community).unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    // Insert two of those posts
    let inserted_post = Post::create(conn, &new_post).unwrap();
    let _inserted_post_again = Post::create(conn, &new_post).unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    // Insert two of those comments
    let inserted_comment = Comment::create(conn, &comment_form, None).unwrap();

    let child_comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let _inserted_child_comment =
      Comment::create(conn, &child_comment_form, Some(&inserted_comment.path)).unwrap();

    let site_aggregates_before_delete = SiteAggregates::read(conn).unwrap();

    // TODO: this is unstable, sometimes it returns 0 users, sometimes 1
    //assert_eq!(0, site_aggregates_before_delete.users);
    assert_eq!(1, site_aggregates_before_delete.communities);
    assert_eq!(2, site_aggregates_before_delete.posts);
    assert_eq!(2, site_aggregates_before_delete.comments);

    // Try a post delete
    Post::delete(conn, inserted_post.id).unwrap();
    let site_aggregates_after_post_delete = SiteAggregates::read(conn).unwrap();
    assert_eq!(1, site_aggregates_after_post_delete.posts);
    assert_eq!(0, site_aggregates_after_post_delete.comments);

    // This shouuld delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(conn, inserted_person.id).unwrap();
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(conn, inserted_community.id).unwrap();
    assert_eq!(1, community_num_deleted);

    // Site should still exist, it can without a site creator.
    let after_delete_creator = SiteAggregates::read(conn);
    assert!(after_delete_creator.is_ok());

    Site::delete(conn, inserted_site.id).unwrap();
    let after_delete_site = SiteAggregates::read(conn);
    assert!(after_delete_site.is_err());

    Instance::delete(conn, inserted_instance.id).unwrap();
  }
}
