#[macro_use]
extern crate bencher;
extern crate redis;

use bencher::Bencher;

use redis::PipelineCommands;

fn get_client() -> redis::Client {
    redis::Client::open("redis://127.0.0.1:6379").unwrap()
}

fn bench_simple_getsetdel(b: &mut Bencher) {
    let client = get_client();
    let con = client.get_connection().unwrap();

    b.iter(|| {
        let key = "test_key";
        redis::cmd("SET").arg(key).arg(42).execute(&con);
        let _: isize = redis::cmd("GET").arg(key).query(&con).unwrap();
        redis::cmd("DEL").arg(key).execute(&con);
    });
}

#[bench]
fn bench_big_pipeline(b: &mut Bencher) {
    let client = get_client();
    let con = client.get_connection().unwrap();

    let data_size = 100;

    b.iter(|| {
        let mut pipe = redis::pipe();
        for x in 0..data_size {
            let test_key = format!("test_{}", x);
            pipe.cmd("SET").arg(test_key).arg(x.to_string()).ignore();
        }
        let _: () = pipe.query(&con).unwrap();
        let mut pipe = redis::pipe();
        for x in 0..data_size {
            let test_key = format!("test_{}", x);
            pipe.cmd("GET").arg(test_key);
        }
        let mut result: Vec<String> = pipe.query(&con).unwrap();
        result.remove(99);
    });
}

#[bench]
fn bench_complex(b: &mut Bencher) {
    let client = get_client();
    let con = client.get_connection().unwrap();

    let data_size = 100;

    b.iter(|| {
        for x in 0..data_size {
            let key: isize = redis::cmd("INCR").arg("id_gen").query(&con).unwrap();
            let key = format!("id_{}", key);
            redis::cmd("SET").arg(key).arg(x.to_string()).execute(&con);
        }
    });
}

#[bench]
fn bench_complex_pipeline(b: &mut Bencher) {
    let client = get_client();
    let con = client.get_connection().unwrap();

    let data_size = 100;

    b.iter(|| {
        let mut id_pipe = redis::pipe();
        for x in 0..data_size {
            id_pipe.cmd("INCR").arg("id_gen");
        }
        let ids: Vec<isize> = id_pipe.query(&con).unwrap();
        let mut set_pipe = redis::pipe();
        for x in 0..data_size {
            let key = format!("id_{}", ids[x]);
            set_pipe.cmd("SET").arg(key).arg(x.to_string());
        }
        let confirmations: Vec<String> = set_pipe.query(&con).unwrap();
    });
}

#[bench]
fn bench_simple_getsetdel_pipeline(b: &mut Bencher) {
    let client = get_client();
    let con = client.get_connection().unwrap();

    b.iter(|| {
        let key = "test_key";
        let _: (usize,) = redis::pipe()
            .cmd("SET")
            .arg(key)
            .arg(42)
            .ignore()
            .cmd("GET")
            .arg(key)
            .cmd("DEL")
            .arg(key)
            .ignore()
            .query(&con)
            .unwrap();
    });
}

fn bench_simple_getsetdel_pipeline_precreated(b: &mut Bencher) {
    let client = get_client();
    let con = client.get_connection().unwrap();
    let key = "test_key";
    let mut pipe = redis::pipe();
    pipe.cmd("SET")
        .arg(key)
        .arg(42)
        .ignore()
        .cmd("GET")
        .arg(key)
        .cmd("DEL")
        .arg(key)
        .ignore();

    b.iter(|| {
        let _: (usize,) = pipe.query(&con).unwrap();
    });
}

fn bench_long_pipeline(b: &mut Bencher) {
    let client = get_client();
    let con = client.get_connection().unwrap();
    let mut pipe = redis::pipe();

    for _ in 0..1_000 {
        pipe.set("foo", "bar").ignore();
    }

    b.iter(|| {
        let _: () = pipe.query(&con).unwrap();
    });
}

fn bench_encode_pipeline(b: &mut Bencher) {
    b.iter(|| {
        let mut pipe = redis::pipe();

        for _ in 0..1_000 {
            pipe.set("foo", "bar").ignore();
        }
        pipe
    });
}

fn bench_encode_pipeline_nested(b: &mut Bencher) {
    b.iter(|| {
        let mut pipe = redis::pipe();

        for _ in 0..200 {
            pipe.set(
                "foo",
                ("bar", 123, b"1231279712", &["test", "test", "test"][..]),
            ).ignore();
        }
        pipe
    });
}

benchmark_group!(
    bench,
    bench_simple_getsetdel,
    bench_simple_getsetdel_pipeline,
    bench_simple_getsetdel_pipeline_precreated,
    bench_long_pipeline,
    bench_encode_pipeline,
    bench_encode_pipeline_nested
);
benchmark_main!(bench);
