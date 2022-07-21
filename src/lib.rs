use crate::parser::{parse_file_get_dto, parse_file_get_invocation_structure};
use jni::objects::{JClass, JObject, JString};
use jni::JNIEnv;
use jni::sys::{jstring, jint};
use std::panic;
use std::sync::{Arc, Mutex};
use std::thread::Result;

mod dto;
mod model;
mod parser_impl;
mod parser;
mod utils;
mod visitor;


#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_io_smartforce_plugin_bitbucket_navigation_parser_RustNativeParser_declarationParseGetJson(env: JNIEnv,
                                                                                                                      class: JClass,
                                                                                                                      repository_id: jint,
                                                                                                                      file_data: JString,
                                                                                                                      path: JString,
                                                                                                                      language: JString,
                                                                                                                      error_callback: JObject) -> jstring {
    /* Setting up panic buffer in order to catch panic messages */
    let old_panic_hook = panic::take_hook();

    let panic_buffer = setup_panic_buffer();

    /* Perform parsing */
    let result = panic::catch_unwind(|| { declaration_parse_get_json(&env, repository_id, file_data, path, language) });

    /* Setting back default panic hook */
    panic::set_hook(old_panic_hook);

    let java_json_string = unwrap_result_log_errors(&env, error_callback, result, panic_buffer);

    /* Extract raw pointer to return. */
    return java_json_string.into_inner();
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_io_smartforce_plugin_bitbucket_navigation_parser_RustNativeParser_invocationParseGetJson(env: JNIEnv,
                                                                                                                     class: JClass,
                                                                                                                     file_data: JString,
                                                                                                                     path: JString,
                                                                                                                     language: JString,
                                                                                                                     error_callback: JObject) -> jstring {
    /* Setting up panic buffer in order to catch panic messages */
    let old_panic_hook = panic::take_hook();

    let panic_buffer = setup_panic_buffer();

    /* Perform parsing */
    let result = panic::catch_unwind(|| { invocation_parse_get_json(&env, file_data, path, language) });

    /* Setting back default panic hook */
    panic::set_hook(old_panic_hook);

    let java_json_string = unwrap_result_log_errors(&env, error_callback, result, panic_buffer);

    /* Extract raw pointer to return. */
    return java_json_string.into_inner();
}


fn declaration_parse_get_json<'lifetime>(env: &'lifetime JNIEnv, repository_id: jint,
                                         file_data: JString, path: JString, language: JString) -> JString<'lifetime> {
    let data: String = env.get_string(file_data).expect("Couldn't get java string. Param name: data").into();
    let path: String = env.get_string(path).expect("Couldn't get java string. Param name: path").into();
    let language: String = env.get_string(language).expect("Couldn't get java string. Param name: language").into();

    let method_dto_vec = parse_file_get_dto(data, repository_id, path, language);
    let json = serde_json::to_string(&method_dto_vec).unwrap();

    env.new_string(json).unwrap()
}

fn invocation_parse_get_json<'lifetime>(env: &'lifetime JNIEnv, file_data: JString,
                                        path: JString, language: JString) -> JString<'lifetime> {
    let data: String = env.get_string(file_data).expect("Couldn't get java string. Param name: data").into();
    let path: String = env.get_string(path).expect("Couldn't get java string. Param name: path").into();
    let language: String = env.get_string(language).expect("Couldn't get java string. Param name: lang").into();

    let invocation_structure = parse_file_get_invocation_structure(data, path, language);
    let json = serde_json::to_string(&invocation_structure).unwrap();

    env.new_string(json).unwrap()
}

fn setup_panic_buffer() -> Arc<Mutex<String>> {
    let global_error_buffer = Arc::new(Mutex::new(String::new()));

    panic::set_hook({
        let global_error_buffer = global_error_buffer.clone();
        Box::new(move |info| {
            let mut global_error_buffer = global_error_buffer.lock().unwrap();
            if let Some(error_message) = info.payload().downcast_ref::<&str>() {
                global_error_buffer.push_str(error_message);
            }
        })
    });

    global_error_buffer
}

fn unwrap_result_log_errors<'lifetime>(env: &'lifetime JNIEnv, error_callback: JObject,
                                       result: Result<JString<'lifetime>>, panic_buffer: Arc<Mutex<String>>) -> JString<'lifetime> {
    return match result {
        Ok(json_string) => json_string,

        Err(_) => {
            let error_message = panic_buffer.lock().unwrap().clone();
            log_error_callback(env, error_callback, error_message);
            env.new_string(String::new()).unwrap()
        }
    };
}

fn log_error_callback(env: &JNIEnv, error_callback: JObject, error_message: String) {
    let java_str = env.new_string(error_message).unwrap();

    env.call_method(
        error_callback,
        "logErrorCallback",
        "(Ljava/lang/String;)V",
        &[java_str.into()],
    ).unwrap();
}

