// @generated
pub mod sf {
    pub mod firehose {
        // @@protoc_insertion_point(attribute:sf.firehose.v2)
        pub mod v2 {
            include!("sf.firehose.v2.rs");
            // @@protoc_insertion_point(sf.firehose.v2)
        }
    }
    // @@protoc_insertion_point(attribute:sf.substreams)
    pub mod substreams {
        include!("sf.substreams.rs");
        // @@protoc_insertion_point(sf.substreams)
        pub mod index {
            // @@protoc_insertion_point(attribute:sf.substreams.index.v1)
            pub mod v1 {
                include!("sf.substreams.index.v1.rs");
                // @@protoc_insertion_point(sf.substreams.index.v1)
            }
        }
        pub mod internal {
            // @@protoc_insertion_point(attribute:sf.substreams.internal.v2)
            pub mod v2 {
                include!("sf.substreams.internal.v2.rs");
                // @@protoc_insertion_point(sf.substreams.internal.v2)
            }
        }
        pub mod rpc {
            // @@protoc_insertion_point(attribute:sf.substreams.rpc.v2)
            pub mod v2 {
                include!("sf.substreams.rpc.v2.rs");
                // @@protoc_insertion_point(sf.substreams.rpc.v2)
            }
        }
        pub mod sink {
            pub mod service {
                // @@protoc_insertion_point(attribute:sf.substreams.sink.service.v1)
                pub mod v1 {
                    include!("sf.substreams.sink.service.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.sink.service.v1)
                }
            }
        }
        // @@protoc_insertion_point(attribute:sf.substreams.v1)
        pub mod v1 {
            include!("sf.substreams.v1.rs");
            // @@protoc_insertion_point(sf.substreams.v1)
        }
    }
}
