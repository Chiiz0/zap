use warp_core::HostId;

use super::{SessionContext, SessionType, SkillPathOrigin};

#[test]
fn skill_origin_keeps_the_connected_ssh_host_without_cwd() {
    let host_id = HostId::new("remote-host".to_owned());
    let session = SessionContext {
        session_type: Some(SessionType::WarpifiedRemote {
            host_id: Some(host_id.clone()),
        }),
        current_working_directory: None,
        is_legacy_ssh: true,
        ..SessionContext::new_for_test()
    };

    assert_eq!(
        session.skill_path_origin(),
        SkillPathOrigin::Remote { host_id }
    );
}

#[test]
fn skill_origin_is_unavailable_while_ssh_host_is_unknown() {
    let session = SessionContext {
        session_type: Some(SessionType::WarpifiedRemote { host_id: None }),
        current_working_directory: Some("/repo".to_owned()),
        ..SessionContext::new_for_test()
    };

    assert_eq!(session.skill_path_origin(), SkillPathOrigin::Unavailable);
}

#[test]
fn skill_origin_does_not_treat_legacy_ssh_as_local() {
    let session = SessionContext {
        session_type: Some(SessionType::Local),
        is_legacy_ssh: true,
        ..SessionContext::new_for_test()
    };

    assert_eq!(session.skill_path_origin(), SkillPathOrigin::Unavailable);
}

#[test]
fn skill_origin_is_unavailable_for_legacy_ssh_without_session_type() {
    let session = SessionContext {
        session_type: None,
        is_legacy_ssh: true,
        ..SessionContext::new_for_test()
    };

    assert_eq!(session.skill_path_origin(), SkillPathOrigin::Unavailable);
}

#[test]
fn skill_origin_keeps_local_skills_without_cwd() {
    let session = SessionContext {
        session_type: Some(SessionType::Local),
        current_working_directory: None,
        ..SessionContext::new_for_test()
    };

    assert_eq!(session.skill_path_origin(), SkillPathOrigin::Local);
}
