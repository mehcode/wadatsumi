// todo: mod decode;
mod execute;
// todo: mod ƒmt;

#[derive(Debug)]
pub enum ArmInstruction {
    BranchExchange,
    BranchLink,
    DataProcessing,
    PsrTransfer,
    Multiply,
    SingleDataTransfer,
    HalfWordSignedTransfer,
    BlockDataTransfer,
    SingleDataSwap,
    SoftwareInterrupt,
}
