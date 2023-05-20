use crate::{parsers::JobDeclaration, utils::Mutex};
use std::sync::Arc;
pub type SendTo = SendTo_<JobDeclaration<'static>, ()>;
use super::SendTo_;
use crate::errors::Error;
use core::convert::TryInto;
use job_declaration_sv2::*;

/// A trait implemented by a downstream to handle SV2 job declaration messages.
pub trait ParseServerJobDeclarationMessages
where
    Self: Sized,
{
    /// Used to parse job declaration message and route to the message's respected handler function
    fn handle_message_job_declaration(
        self_: Arc<Mutex<Self>>,
        message_type: u8,
        payload: &mut [u8],
    ) -> Result<SendTo, Error> {
        match (message_type, payload).try_into() {
            Ok(JobDeclaration::AllocateMiningJobTokenSuccess(message)) => self_
                .safe_lock(|x| x.handle_allocate_mining_job_token_sucess(message))
                .map_err(|e| crate::Error::PoisonLock(e.to_string()))?,
            Ok(JobDeclaration::CommitMiningJobSuccess(message)) => self_
                .safe_lock(|x| x.handle_commit_mining_job_success(message))
                .map_err(|e| crate::Error::PoisonLock(e.to_string()))?,
            Ok(_) => todo!(),
            Err(e) => Err(e),
        }
    }
    /// When upstream send AllocateMiningJobTokenSuccess self should use the received token to
    /// negotiate the next job
    ///
    /// "[`job_declaration_sv2::AllocateMiningJobToken`]"
    fn handle_allocate_mining_job_token_sucess(
        &mut self,
        message: AllocateMiningJobTokenSuccess,
    ) -> Result<SendTo, Error>;

    // When upstream send CommitMiningJobSuccess if the token is different from the one negotiated
    // self must use the new token to refer to the committed job
    fn handle_commit_mining_job_success(
        &mut self,
        message: CommitMiningJobSuccess,
    ) -> Result<SendTo, Error>;
}
pub trait ParseClientJobDeclarationMessages
where
    Self: Sized,
{
    fn handle_message_job_declaration(
        self_: Arc<Mutex<Self>>,
        message_type: u8,
        payload: &mut [u8],
    ) -> Result<SendTo, Error> {
        match (message_type, payload).try_into() {
            Ok(JobDeclaration::AllocateMiningJobToken(message)) => self_
                .safe_lock(|x| x.handle_allocate_mining_job(message))
                .map_err(|e| crate::Error::PoisonLock(e.to_string()))?,
            Ok(JobDeclaration::CommitMiningJob(message)) => self_
                .safe_lock(|x| x.handle_commit_mining_job(message))
                .map_err(|e| crate::Error::PoisonLock(e.to_string()))?,
            Ok(_) => todo!(),
            Err(e) => Err(e),
        }
    }
    fn handle_allocate_mining_job(
        &mut self,
        message: AllocateMiningJobToken,
    ) -> Result<SendTo, Error>;
    fn handle_commit_mining_job(&mut self, message: CommitMiningJob) -> Result<SendTo, Error>;
}
