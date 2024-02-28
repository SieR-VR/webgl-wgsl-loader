use serde_derive::{Deserialize, Serialize};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use naga::{back::glsl, proc::{BoundsCheckPolicies, BoundsCheckPolicy}, valid::{Capabilities, ValidationFlags}};

#[napi(object)]
#[derive(Serialize, Deserialize, Default)]
struct CompileResult {
    pub vertex_shader: Option<String>,
    pub fragment_shader: Option<String>,
    pub compute_shader: Option<String>,
}

#[napi]
fn compile_wgsl(wgsl_string: String) -> Result<CompileResult, Status> {
    let result = naga::front::wgsl::parse_str(&wgsl_string);

    let module = match result {
        Ok(v) => v,
        Err(ref e) => {
            let message = format!(
                "Could not parse WGSL\n{}",
                e.emit_to_string(&wgsl_string)
            );
            return Err(Error::new(
                Status::GenericFailure,
                message
            ));
        }
    };

    let info = match naga::valid::Validator::new(ValidationFlags::default(), Capabilities::default())
        .validate(&module)
    {
        Ok(info) => info,
        Err(error) => {
            println!("Validation error: {:?}", error);
            return Err(Error::new(
                Status::GenericFailure,
                "Generating glsl output requires validation to \
                succeed, and it failed in a previous step",
            ));
        }
    };

    let mut glsl_options = glsl::Options::default();
    glsl_options.version = glsl::Version::new_gles(300);

    let bound_check_policies = BoundsCheckPolicies {
        binding_array: BoundsCheckPolicy::Restrict,
        buffer: BoundsCheckPolicy::Unchecked,
        image_load: BoundsCheckPolicy::Unchecked,
        image_store: BoundsCheckPolicy::Unchecked,
        index: BoundsCheckPolicy::Unchecked,
    };
    let mut result = CompileResult::default();

    for entry in module.entry_points.iter() {
        let pipeline_options = glsl::PipelineOptions {
            entry_point: entry.name.clone(),
            shader_stage: entry.stage,
            multiview: None,
        };

        let mut buffer = String::new();
        let mut writer = glsl::Writer::new(
            &mut buffer,
            &module,
            &info,
            &glsl_options,
            &pipeline_options,
            bound_check_policies,
        );

        match &mut writer {
            Ok(writer) => {
                let reflection_info = match writer.write() {
                    Ok(reflection_info) => reflection_info,
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(Error::new(
                            Status::GenericFailure,
                            "Error writing glsl output",
                        ));
                    }
                };

                match entry.stage {
                    naga::ShaderStage::Vertex => {
                        result.vertex_shader = Some(buffer);
                    }
                    naga::ShaderStage::Fragment => {
                        result.fragment_shader = Some(buffer);
                    }
                    naga::ShaderStage::Compute => {
                        result.compute_shader = Some(buffer);
                    }
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(Error::new(
                    Status::GenericFailure,
                    "Error writing glsl output",
                ));
            }
        }
    }

    Ok(result)
}
