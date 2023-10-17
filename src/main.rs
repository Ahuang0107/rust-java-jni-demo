use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jvm_args = jni::InitArgsBuilder::new()
        .version(jni::JNIVersion::V8)
        .option("-Djava.class.path=jar/java-sample-lib-1.0-SNAPSHOT.jar")
        .build()
        .unwrap();
    let jvm = std::sync::Arc::new(jni::JavaVM::new(jvm_args)?);

    let host = "0.0.0.0";
    let port = 8090;
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .app_data(web::Data::new(jvm.clone()))
            .service(web::scope("/public").service(public))
    })
    .bind((host, port))?
    .run()
    .await?;

    Ok(())
}

#[get("/{name}")]
pub async fn public(
    jvm: web::Data<std::sync::Arc<jni::JavaVM>>,
    name: web::Path<String>,
) -> impl Responder {
    let mut env = jvm.attach_current_thread().unwrap();
    let name = env.new_string(name.as_ref()).unwrap();
    let stamp_util_class = env.find_class("org/example/Sample").unwrap();
    // 具体sig见https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/types.html#type_signatures
    let result = env
        .call_static_method(
            stamp_util_class,
            "hello",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[jni::objects::JValue::Object(&name)],
        )
        .unwrap();
    let result = jni::objects::JString::from(result.l().unwrap());
    let result: String = env.get_string(&result).unwrap().into();
    HttpResponse::Ok().body(result)
}
