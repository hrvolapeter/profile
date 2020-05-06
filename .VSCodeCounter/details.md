# Details

Date : 2020-05-02 11:42:45

Directory /Users/peterhrvola/dev/scheduler

Total : 56 files,  3182 codes, 64 comments, 473 blanks, all 3719 lines

[summary](results.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [.dockerignore](/.dockerignore) | Ignore | 1 | 0 | 0 | 1 |
| [Cargo.toml](/Cargo.toml) | TOML | 12 | 0 | 1 | 13 |
| [Dockerfile](/Dockerfile) | Dockerfile | 10 | 0 | 3 | 13 |
| [components/benchmark/Cargo.toml](/components/benchmark/Cargo.toml) | TOML | 14 | 1 | 2 | 17 |
| [components/benchmark/src/lib.rs](/components/benchmark/src/lib.rs) | Rust | 25 | 0 | 4 | 29 |
| [components/benchmark/src/main.rs](/components/benchmark/src/main.rs) | Rust | 5 | 0 | 1 | 6 |
| [components/cost_flow/Cargo.toml](/components/cost_flow/Cargo.toml) | TOML | 8 | 1 | 3 | 12 |
| [components/cost_flow/src/bfs.rs](/components/cost_flow/src/bfs.rs) | Rust | 62 | 3 | 13 | 78 |
| [components/cost_flow/src/ford_fulkerson.rs](/components/cost_flow/src/ford_fulkerson.rs) | Rust | 47 | 3 | 9 | 59 |
| [components/cost_flow/src/lib.rs](/components/cost_flow/src/lib.rs) | Rust | 408 | 4 | 63 | 475 |
| [components/cost_flow/src/minimum_cost_flow.rs](/components/cost_flow/src/minimum_cost_flow.rs) | Rust | 157 | 0 | 26 | 183 |
| [components/cpu/Cargo.toml](/components/cpu/Cargo.toml) | TOML | 8 | 1 | 3 | 12 |
| [components/cpu/src/lib.rs](/components/cpu/src/lib.rs) | Rust | 20 | 0 | 4 | 24 |
| [components/cpu/src/main.rs](/components/cpu/src/main.rs) | Rust | 5 | 0 | 2 | 7 |
| [components/disk/Cargo.toml](/components/disk/Cargo.toml) | TOML | 9 | 1 | 3 | 13 |
| [components/disk/src/lib.rs](/components/disk/src/lib.rs) | Rust | 68 | 0 | 12 | 80 |
| [components/disk/src/main.rs](/components/disk/src/main.rs) | Rust | 19 | 0 | 5 | 24 |
| [components/measure/Cargo.toml](/components/measure/Cargo.toml) | TOML | 21 | 1 | 3 | 25 |
| [components/measure/src/application_profile.rs](/components/measure/src/application_profile.rs) | Rust | 53 | 1 | 5 | 59 |
| [components/measure/src/bpf/mod.rs](/components/measure/src/bpf/mod.rs) | Rust | 46 | 0 | 9 | 55 |
| [components/measure/src/bpf/profile.rs](/components/measure/src/bpf/profile.rs) | Rust | 94 | 0 | 11 | 105 |
| [components/measure/src/lib.rs](/components/measure/src/lib.rs) | Rust | 76 | 1 | 10 | 87 |
| [components/measure/src/main.rs](/components/measure/src/main.rs) | Rust | 57 | 0 | 9 | 66 |
| [components/measure/src/perf/mod.rs](/components/measure/src/perf/mod.rs) | Rust | 48 | 0 | 9 | 57 |
| [components/measure/src/perf/profile.rs](/components/measure/src/perf/profile.rs) | Rust | 51 | 0 | 5 | 56 |
| [components/measure/src/pmap/mod.rs](/components/measure/src/pmap/mod.rs) | Rust | 49 | 0 | 9 | 58 |
| [components/memory/Cargo.toml](/components/memory/Cargo.toml) | TOML | 10 | 1 | 3 | 14 |
| [components/memory/src/lib.rs](/components/memory/src/lib.rs) | Rust | 40 | 3 | 8 | 51 |
| [components/memory/src/main.rs](/components/memory/src/main.rs) | Rust | 10 | 0 | 3 | 13 |
| [components/network/Cargo.toml](/components/network/Cargo.toml) | TOML | 7 | 1 | 3 | 11 |
| [components/network/src/lib.rs](/components/network/src/lib.rs) | Rust | 16 | 0 | 3 | 19 |
| [components/network/src/main.rs](/components/network/src/main.rs) | Rust | 4 | 0 | 2 | 6 |
| [components/scheduler/Cargo.toml](/components/scheduler/Cargo.toml) | TOML | 28 | 1 | 3 | 32 |
| [components/scheduler/build.rs](/components/scheduler/build.rs) | Rust | 5 | 0 | 1 | 6 |
| [components/scheduler/src/main.rs](/components/scheduler/src/main.rs) | Rust | 65 | 0 | 10 | 75 |
| [components/scheduler/src/rpc/mod.rs](/components/scheduler/src/rpc/mod.rs) | Rust | 142 | 0 | 22 | 164 |
| [components/scheduler/src/scheduler/mod.rs](/components/scheduler/src/scheduler/mod.rs) | Rust | 32 | 0 | 4 | 36 |
| [components/scheduler/src/scheduler/resource_profile.rs](/components/scheduler/src/scheduler/resource_profile.rs) | Rust | 107 | 3 | 19 | 129 |
| [components/scheduler/src/scheduler/scheduler.rs](/components/scheduler/src/scheduler/scheduler.rs) | Rust | 218 | 35 | 35 | 288 |
| [components/scheduler/src/scheduler/server.rs](/components/scheduler/src/scheduler/server.rs) | Rust | 31 | 0 | 5 | 36 |
| [components/scheduler/src/scheduler/task.rs](/components/scheduler/src/scheduler/task.rs) | Rust | 120 | 1 | 13 | 134 |
| [components/scheduler/src/scheduler/virtual_resource.rs](/components/scheduler/src/scheduler/virtual_resource.rs) | Rust | 17 | 0 | 4 | 21 |
| [components/scheduler/src/webui/handlers.rs](/components/scheduler/src/webui/handlers.rs) | Rust | 164 | 0 | 20 | 184 |
| [components/scheduler/src/webui/mod.rs](/components/scheduler/src/webui/mod.rs) | Rust | 68 | 0 | 11 | 79 |
| [components/scheduler/src/webui/pages/footer.hbs](/components/scheduler/src/webui/pages/footer.hbs) | Handlebars | 8 | 0 | 0 | 8 |
| [components/scheduler/src/webui/pages/graph.hbs](/components/scheduler/src/webui/pages/graph.hbs) | Handlebars | 48 | 0 | 8 | 56 |
| [components/scheduler/src/webui/pages/header.hbs](/components/scheduler/src/webui/pages/header.hbs) | Handlebars | 133 | 0 | 18 | 151 |
| [components/scheduler/src/webui/pages/index.hbs](/components/scheduler/src/webui/pages/index.hbs) | Handlebars | 3 | 0 | 2 | 5 |
| [components/scheduler/src/webui/pages/server.hbs](/components/scheduler/src/webui/pages/server.hbs) | Handlebars | 94 | 0 | 3 | 97 |
| [components/scheduler/src/webui/pages/task.hbs](/components/scheduler/src/webui/pages/task.hbs) | Handlebars | 104 | 0 | 6 | 110 |
| [components/scheduler_agent/Cargo.toml](/components/scheduler_agent/Cargo.toml) | TOML | 22 | 1 | 4 | 27 |
| [components/scheduler_agent/build.rs](/components/scheduler_agent/build.rs) | Rust | 5 | 0 | 1 | 6 |
| [components/scheduler_agent/src/main.rs](/components/scheduler_agent/src/main.rs) | Rust | 114 | 0 | 14 | 128 |
| [components/scheduler_agent/src/task.rs](/components/scheduler_agent/src/task.rs) | Rust | 118 | 0 | 10 | 128 |
| [components/scheduler_proto/scheduler.proto](/components/scheduler_proto/scheduler.proto) | Protocol Buffers | 71 | 0 | 13 | 84 |
| [rustfmt.toml](/rustfmt.toml) | TOML | 5 | 1 | 1 | 7 |

[summary](results.md)