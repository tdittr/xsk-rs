#[allow(dead_code)]
mod setup;
use std::convert::TryInto;

use setup::{PacketGenerator, Xsk, XskConfig};

use serial_test::serial;
use xsk_rs::config::{QueueSize, SocketConfig, UmemConfig};

const FQ_SIZE: u32 = 4;
const FRAME_COUNT: u32 = 32;

fn build_configs() -> (UmemConfig, SocketConfig) {
    let umem_config = UmemConfig::builder()
        .fill_queue_size(QueueSize::new(FQ_SIZE).unwrap())
        .build()
        .unwrap();

    let socket_config = SocketConfig::default();

    (umem_config, socket_config)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn producing_fq_size_frames_is_ok() {
    fn test(dev1: (Xsk, PacketGenerator), _dev2: (Xsk, PacketGenerator)) {
        let mut xsk1 = dev1.0;

        assert_eq!(unsafe { xsk1.fq.produce(&xsk1.descs[..4]) }, 4);
    }

    build_configs_and_run_test(test).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn producing_more_than_fq_size_frames_fails() {
    fn test(dev1: (Xsk, PacketGenerator), _dev2: (Xsk, PacketGenerator)) {
        let mut xsk1 = dev1.0;

        assert_eq!(unsafe { xsk1.fq.produce(&xsk1.descs[..5]) }, 0);
    }

    build_configs_and_run_test(test).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn produce_frames_repeatedly_until_full() {
    fn test(dev1: (Xsk, PacketGenerator), _dev2: (Xsk, PacketGenerator)) {
        let mut xsk1 = dev1.0;

        assert_eq!(unsafe { xsk1.fq.produce(&xsk1.descs[..2]) }, 2);
        assert_eq!(unsafe { xsk1.fq.produce(&xsk1.descs[2..3]) }, 1);
        assert_eq!(unsafe { xsk1.fq.produce(&xsk1.descs[3..8]) }, 0);
        assert_eq!(unsafe { xsk1.fq.produce(&xsk1.descs[3..4]) }, 1);
    }

    build_configs_and_run_test(test).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn produce_one_is_ok() {
    fn test(dev1: (Xsk, PacketGenerator), _dev2: (Xsk, PacketGenerator)) {
        let mut xsk1 = dev1.0;

        assert_eq!(unsafe { xsk1.fq.produce_one(&xsk1.descs[0]) }, 1);
    }

    build_configs_and_run_test(test).await
}

async fn build_configs_and_run_test<F>(test: F)
where
    F: Fn((Xsk, PacketGenerator), (Xsk, PacketGenerator)) + Send + 'static,
{
    let (dev1_umem_config, dev1_socket_config) = build_configs();
    let (dev2_umem_config, dev2_socket_config) = build_configs();

    setup::run_test(
        XskConfig {
            frame_count: FRAME_COUNT.try_into().unwrap(),
            umem_config: dev1_umem_config,
            socket_config: dev1_socket_config,
        },
        XskConfig {
            frame_count: FRAME_COUNT.try_into().unwrap(),
            umem_config: dev2_umem_config,
            socket_config: dev2_socket_config,
        },
        test,
    )
    .await;
}
