pub mod alterconfigs_request;
pub mod deletetopics_request;
pub mod describeconfigs_request;
pub mod findcoordinator_request;
pub mod listoffsets_request;
pub mod metadata_request;
pub mod offsetfetch_request;

pub enum ResourceTypes {
    Unknown = 0,
    Any = 1,
    Topic = 2,
    Group = 3,
    Cluster = 4,
    Broker = 5,
}
