use serial_test::serial;
use xsk_rs::{socket::Config as SocketConfig, umem::Config as UmemConfig};

mod setup;
use setup::{UmemConfigBuilder, Xsk};
use std::collections::VecDeque;
use xsk_rs::umem::Frame;

fn build_configs() -> (Option<UmemConfig>, Option<SocketConfig>) {
    let umem_config = UmemConfigBuilder {
        frame_count: 8,
        fill_queue_size: 4,
        ..UmemConfigBuilder::default()
    }
    .build();

    (Some(umem_config), None)
}

#[tokio::test]
#[serial]
async fn fill_queue_produce_tx_size_frames() {
    fn test_fn(mut dev1: Xsk, _dev2: Xsk) {
        let mut frames: VecDeque<Frame> = dev1.frames.into();
        let old_len = frames.len();
        dev1.fill_q.produce(&mut frames);

        assert_eq!(old_len - frames.len(), 4);
    }

    let (dev1_umem_config, dev1_socket_config) = build_configs();
    let (dev2_umem_config, dev2_socket_config) = build_configs();

    setup::run_test(
        dev1_umem_config,
        dev1_socket_config,
        dev2_umem_config,
        dev2_socket_config,
        test_fn,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn fill_queue_produce_gt_tx_size_frames() {
    fn test_fn(mut dev1: Xsk, _dev2: Xsk) {
        let mut frames_to_fill = VecDeque::with_capacity(5);
        for _ in 0..5 {
            frames_to_fill.push_back(dev1.frames.pop().unwrap());
        }
        dev1.fill_q.produce(&mut frames_to_fill);
        assert!(frames_to_fill.is_empty());
    }

    let (dev1_umem_config, dev1_socket_config) = build_configs();
    let (dev2_umem_config, dev2_socket_config) = build_configs();

    setup::run_test(
        dev1_umem_config,
        dev1_socket_config,
        dev2_umem_config,
        dev2_socket_config,
        test_fn,
    )
    .await;
}

// ToDo: Fix test
/*
#[tokio::test]
#[serial]
async fn fill_queue_produce_frames_until_full() {
    fn test_fn(mut dev1: Xsk, _dev2: Xsk) {
        let frame_descs = dev1.frames;

        assert_eq!(unsafe { dev1.fill_q.produce(&frame_descs[..2]) }, 2);
        assert_eq!(unsafe { dev1.fill_q.produce(&frame_descs[2..3]) }, 1);
        assert_eq!(unsafe { dev1.fill_q.produce(&frame_descs[3..8]) }, 0);
        assert_eq!(unsafe { dev1.fill_q.produce(&frame_descs[3..4]) }, 1);
    }

    let (dev1_umem_config, dev1_socket_config) = build_configs();
    let (dev2_umem_config, dev2_socket_config) = build_configs();

    setup::run_test(
        dev1_umem_config,
        dev1_socket_config,
        dev2_umem_config,
        dev2_socket_config,
        test_fn,
    )
    .await;
}
*/
