///
/// SubnetListRecord
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct SubnetListRecord {
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub subnets: Vec<Vec<u8>>,
}

///
/// SubnetRecord
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct SubnetRecord {
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub membership: Vec<Vec<u8>>,
    #[prost(enumeration = "SubnetType", tag = "15")]
    pub subnet_type: i32,
    #[prost(enumeration = "CanisterCyclesCostSchedule", tag = "30")]
    pub canister_cycles_cost_schedule: i32,
}

///
/// SubnetType
///
#[derive(Clone, Copy, Debug, prost::Enumeration, Eq, PartialEq)]
#[repr(i32)]
pub enum SubnetType {
    Unspecified = 0,
    Application = 1,
    System = 2,
    VerifiedApplication = 4,
    CloudEngine = 5,
}

///
/// CanisterCyclesCostSchedule
///
#[derive(Clone, Copy, Debug, prost::Enumeration, Eq, PartialEq)]
#[repr(i32)]
pub enum CanisterCyclesCostSchedule {
    Unspecified = 0,
    Normal = 1,
    Free = 2,
}
