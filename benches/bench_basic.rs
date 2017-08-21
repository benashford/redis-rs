#![feature(test)]
extern crate test;
extern crate redis;

use test::Bencher;

fn get_client() -> redis::Client {
    redis::Client::open("redis://127.0.0.1:6379").unwrap()
}

#[bench]
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

    let data_size = 1000;

    b.iter(|| {
        let mut pipe = redis::pipe();
        for x in 0..data_size {
            let test_key = format!("test_{}", x);
            pipe.cmd("SET").arg(test_key).arg(x.to_string()).ignore();
        }
        let _:() = pipe.query(&con).unwrap();
        let mut pipe = redis::pipe();
        for x in 0..data_size {
            let test_key = format!("test_{}", x);
            pipe.cmd("GET").arg(test_key);
        }
        let mut result:Vec<String> = pipe.query(&con).unwrap();
        result.remove(999);
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

#[bench]
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
