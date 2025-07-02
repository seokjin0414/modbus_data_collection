use crate::service::server::server_init::server_initializer;
use crate::service::utils::setup_log_file::setup_log_file;
use anyhow::{anyhow, Result};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod model {
    pub mod gems_3005 {
        pub mod data_models;
        pub mod gems_3500_memory_map_models;
    }

    pub mod modbus {
        pub mod modbus_register_models;
    }
    pub mod iaq {
        pub mod data_models;
    }

    pub mod heat {
        pub mod data_models;
    }

    pub mod gas {
        pub mod data_models;
    }
}

mod service {
    pub mod collect {
        pub mod gas;
        pub mod gems_3500_modbus;
        pub mod heat;
    }

    pub mod read {
        pub mod read_from_addr;
        pub mod read_from_register;
        pub mod interpret_modbus_register;

        pub mod iaq {
            pub mod util_funcs;
        }
    }

    pub mod server {
        pub mod get_state;
        pub mod health_check;
        pub mod server_init;
        pub mod udp_listener;
    }

    pub mod task {
        pub mod common_scheduling;
        pub mod task_init;
    }

    pub mod utils {
        pub mod create_time;
        pub mod setup_log_file;
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // let mut filter: EnvFilter =
    //     EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    // filter = filter
    //     .add_directive("bems-svr=info".parse()?)
    //     .add_directive("rustls=off".parse()?)
    //     .add_directive("aws_config=off".parse()?);

    // tracing_subscriber::fmt()
    //     .with_ansi(false) // disable colored output
    //     // .with_target(false) // disable target display
    //     .with_env_filter(filter)
    //     .init();

    // 콘솔 로그: info 이상
    let stdout_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(std::io::stdout) // stdout으로 info/warn/error
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    // 파일 로그: error만
    let error_log_file = setup_log_file()?;
    let file_layer = fmt::layer()
        .with_writer(error_log_file)
        .with_ansi(false)
        .with_filter(EnvFilter::new("error"));

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    let result = server_initializer().await.map_err(|e| anyhow!("{:?}", e))?;

    info!("Server successfully terminated: {}", result);
    Ok(())
}
