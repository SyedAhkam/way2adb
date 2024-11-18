use ashpd::desktop::{
    screencast::{CursorMode, Screencast, SourceType, Stream as AshStream},
    PersistMode,
};
use std::os::fd::OwnedFd;

pub async fn open_portal() -> ashpd::Result<(AshStream, OwnedFd)> {
    let proxy = Screencast::new().await?;
    let session = proxy.create_session().await?;
    proxy
        .select_sources(
            &session,
            CursorMode::Embedded,
            SourceType::Monitor.into(),
            false,
            None,
            PersistMode::DoNot,
        )
        .await?;

    let response = proxy.start(&session, None).await?.response()?;
    let stream = response.streams().first().unwrap().to_owned();
    println!("{:?}", stream.size());

    let fd = proxy.open_pipe_wire_remote(&session).await?;

    Ok((stream, fd))
}
