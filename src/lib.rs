#[macro_use]
extern crate bitflags;
extern crate num_traits;
extern crate spirv_headers;

use num_traits::cast::FromPrimitive;

pub mod convert;
pub mod ffi;
pub mod types;

pub fn ffi_to_string(ffi: *const ::std::os::raw::c_char) -> String {
    if ffi.is_null() {
        String::new()
    } else {
        let c_str: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(ffi) };
        let str_slice: &str = match c_str.to_str() {
            Ok(c_str) => c_str,
            Err(_) => &"",
        };
        str_slice.to_owned()
    }
}

impl Default for ffi::SpvReflectShaderModule {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for ffi::SpvReflectDescriptorSet {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[derive(Default, Clone)]
pub struct ShaderModule {
    module: Option<ffi::SpvReflectShaderModule>,
}

impl ShaderModule {
    pub fn get_code_size(&self) -> usize {
        match self.module {
            Some(module) => unsafe { ffi::spvReflectGetCodeSize(&module) as usize },
            None => 0,
        }
    }

    pub fn get_code_slice(&self) -> &[u32] {
        let code_size = self.get_code_size();
        let module = self.module.unwrap();
        unsafe { std::slice::from_raw_parts(ffi::spvReflectGetCode(&module), code_size / 4) }
    }

    pub fn get_generator(&self) -> types::ReflectGenerator {
        match self.module {
            Some(module) => convert::ffi_to_generator(module.generator),
            None => types::ReflectGenerator::Unknown,
        }
    }

    pub fn get_entry_point_name(&self) -> String {
        match self.module {
            Some(module) => ffi_to_string(module.entry_point_name),
            None => String::new(),
        }
    }

    pub fn get_shader_stage(&self) -> types::ReflectShaderStageFlags {
        match self.module {
            Some(module) => convert::ffi_to_shader_stage_flags(module.shader_stage),
            None => types::ReflectShaderStageFlags::UNDEFINED,
        }
    }

    

    

    pub fn get_source_language(&self) -> spirv_headers::SourceLanguage {
        match self.module {
            Some(module) => match spirv_headers::SourceLanguage::from_i32(module.source_language) {
                Some(language) => language,
                None => spirv_headers::SourceLanguage::Unknown,
            },
            None => spirv_headers::SourceLanguage::Unknown,
        }
    }

    pub fn get_source_language_version(&self) -> u32 {
        match self.module {
            Some(module) => module.source_language_version,
            None => 0,
        }
    }

    pub fn get_source_file(&self) -> String {
        match self.module {
            Some(module) => ffi_to_string(module.source_file),
            None => String::new(),
        }
    }

    pub fn get_source_text(&self) -> String {
        match self.module {
            Some(module) => ffi_to_string(module.source_source),
            None => String::new(),
        }
    }

    pub fn get_spirv_execution_model(&self) -> spirv_headers::ExecutionModel {
        match self.module {
            Some(module) => {
                match spirv_headers::ExecutionModel::from_i32(module.spirv_execution_model) {
                    Some(model) => model,
                    None => spirv_headers::ExecutionModel::Vertex,
                }
            }
            None => spirv_headers::ExecutionModel::Vertex,
        }
    }

    /*

    pub entry_point_id: u32,
    pub entry_point_count: u32,
    pub entry_points: *mut SpvReflectEntryPoint,
    pub descriptor_binding_count: u32,
    pub descriptor_bindings: *mut SpvReflectDescriptorBinding,
    pub descriptor_set_count: u32,
    pub descriptor_sets: [SpvReflectDescriptorSet; 64usize],
    pub input_variable_count: u32,
    pub input_variables: *mut SpvReflectInterfaceVariable,
    pub output_variable_count: u32,
    pub output_variables: *mut SpvReflectInterfaceVariable,
    pub push_constant_block_count: u32,
    pub push_constant_blocks: *mut SpvReflectBlockVariable,
}
*/

    pub fn descriptor_set_count(&self) -> Result<u32, &str> {
        match self.module {
            Some(module) => {
                let mut count: u32 = 0;
                let result = unsafe {
                    ffi::spvReflectEnumerateDescriptorSets(
                        &module,
                        &mut count,
                        ::std::ptr::null_mut(),
                    )
                };
                match result {
                    ffi::SpvReflectResult_SPV_REFLECT_RESULT_SUCCESS => Ok(count),
                    _ => Err(convert::result_to_string(result)),
                }
            }
            None => Ok(0),
        }
    }

    pub fn descriptor_sets(&self) -> Result<Vec<types::ReflectDescriptorSet>, &str> {
        let count = self.descriptor_set_count()?;
        if let Some(ref module) = self.module {
            if count > 0 {
                let mut ffi_sets: Vec<*mut ffi::SpvReflectDescriptorSet> =
                    vec![::std::ptr::null_mut(); count as usize];
                let result = unsafe {
                    let mut out_count: u32 = count;
                    ffi::spvReflectEnumerateDescriptorSets(
                        module,
                        &mut out_count,
                        ffi_sets.as_mut_ptr(),
                    )
                };
                match result {
                    ffi::SpvReflectResult_SPV_REFLECT_RESULT_SUCCESS => {
                        let mut sets = Vec::new();
                        for ffi_set in ffi_sets {
                            let ffi_set_ref = unsafe { &*ffi_set };
                            let mut bindings: Vec<
                                types::ReflectDescriptorBinding,
                            > = Vec::with_capacity(ffi_set_ref.binding_count as usize);
                            let ffi_bindings = unsafe {
                                std::slice::from_raw_parts(
                                    ffi_set_ref.bindings,
                                    ffi_set_ref.binding_count as usize,
                                )
                            };
                            for ffi_binding in ffi_bindings {
                                let ffi_binding_ref = unsafe { &**ffi_binding };
                                bindings.push(types::ReflectDescriptorBinding {
                                    spirv_id: ffi_binding_ref.spirv_id,
                                    name: ffi_to_string(ffi_binding_ref.name),
                                    binding: ffi_binding_ref.binding,
                                    input_attachment_index: ffi_binding_ref.input_attachment_index,
                                    set: ffi_binding_ref.set,
                                    descriptor_type: convert::ffi_to_descriptor_type(
                                        ffi_binding_ref.descriptor_type,
                                    ),
                                    resource_type: convert::ffi_to_resource_type(
                                        ffi_binding_ref.resource_type,
                                    ),
                                    image: convert::ffi_to_image_traits(ffi_binding_ref.image),
                                    block: convert::ffi_to_block_variable(ffi_binding_ref.block),
                                    array: convert::ffi_to_binding_array_traits(
                                        ffi_binding_ref.array,
                                    ),
                                    count: ffi_binding_ref.count,
                                    uav_counter_id: ffi_binding_ref.uav_counter_id,
                                    uav_counter_binding: match ffi_binding_ref
                                        .uav_counter_binding
                                        .is_null()
                                    {
                                        true => None,
                                        false => None,
                                    },
                                    type_description: match ffi_binding_ref
                                        .type_description
                                        .is_null()
                                    {
                                        true => None,
                                        false => None,
                                    },

                                    //ffi_to_type_description()
                                    //uav_counter_binding: convert::ffi_to_uav_counter_binding(ff_binding_ref.uav_counter_binding),
                                    //type_description: convert::ffi_to_uav_counter_binding(ff_binding_ref.uav_counter_binding),
                                    word_offset: (
                                        ffi_binding_ref.word_offset.binding,
                                        ffi_binding_ref.word_offset.set,
                                    ),
                                });
                            }
                            sets.push(types::descriptor::ReflectDescriptorSet {
                                set: ffi_set_ref.set,
                                bindings,
                            });
                        }
                        Ok(sets)
                    }
                    _ => Err(convert::result_to_string(result)),
                }
            } else {
                // No descriptor sets
                Ok(Vec::new())
            }
        } else {
            // Invalid shader module
            Ok(Vec::new())
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        if let Some(mut module) = self.module {
            unsafe {
                ffi::spvReflectDestroyShaderModule(&mut module);
            }
        }
    }
}

pub fn create_shader_module(spv_data: &[u8]) -> Result<ShaderModule, &str> {
    let mut module: ffi::SpvReflectShaderModule = unsafe { std::mem::zeroed() };
    let result: ffi::SpvReflectResult = unsafe {
        ffi::spvReflectCreateShaderModule(
            spv_data.len(),
            spv_data.as_ptr() as *const std::os::raw::c_void,
            &mut module,
        )
    };
    match result {
        ffi::SpvReflectResult_SPV_REFLECT_RESULT_SUCCESS => Ok(ShaderModule {
            module: Some(module),
        }),
        _ => Err(convert::result_to_string(result)),
    }
}
