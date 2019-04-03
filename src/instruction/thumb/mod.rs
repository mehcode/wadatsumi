mod execute;
// todo: mod decode;
// todo: mod ƒmt;

#[derive(Debug)]
pub enum ThumbInstruction {
    MoveShiftedRegister,
    AddSub,
    MoveCompareAddSubtractImmediate,
    AluOps,
    HiRegisterOpsBranchExchange,
    LoadPcRelative,
    LoadStoreWithRegisterOffset,
    LoadStoreSignExtended,
    LoadStoreWithImmediateOffset,
    LoadStoreHalfWord,
    LoadStoreSpRelative,
    GetRelativeAddress,
    AddOffsetToSp,
    PushPop,
    MultipleLoadStore,
    ConditionalBranch,
    UnconditionalBranch,
    LongBranchWithLink,
    SoftwareInterrupt,
}
