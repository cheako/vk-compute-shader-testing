use std::{ffi::CString, io::Read};

use ash::{extensions::khr, vk};
use nom::Parser;
use rand::Rng;

fn main() {
    let app_name = CString::new("Compute Shader Testing").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0, 0, 0, 1))
        .engine_name(&app_name)
        .engine_version(vk::make_api_version(0, 0, 0, 1))
        .api_version(vk::API_VERSION_1_3);

    let entry = unsafe { ash::Entry::load() }.unwrap();

    let mut enabled_layer_names = vec![];

    if cfg!(not(feature = "no_validation")) {
        enabled_layer_names.push("VK_LAYER_KHRONOS_validation\0".as_ptr().cast())
    }

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&enabled_layer_names);

    let instance = unsafe { entry.create_instance(&create_info, None) }.unwrap();

    let physical_devices = unsafe { instance.enumerate_physical_devices() }.unwrap();

    let physical_device = physical_devices
        .iter()
        .find_map(|physical_device| {
            let prop = unsafe { instance.get_physical_device_properties(*physical_device) };

            if prop.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                Some(physical_device)
            } else {
                None
            }
        })
        .unwrap();

    // I'm just guessing here.
    let enabled_extension_names = [
        khr::Maintenance4::name().as_ptr(),
        khr::PushDescriptor::name().as_ptr(),
        "VK_NV_compute_shader_derivatives\0".as_ptr().cast(),
    ];

    let mut features0 = vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
        .descriptor_binding_partially_bound(true)
        .descriptor_binding_variable_descriptor_count(true)
        .descriptor_binding_update_unused_while_pending(true)
        .descriptor_binding_sampled_image_update_after_bind(true);
    let mut features1 = vk::PhysicalDeviceVulkan12Features::builder()
        .runtime_descriptor_array(true)
        .buffer_device_address(true);
    let mut features2 = vk::PhysicalDeviceComputeShaderDerivativesFeaturesNV::builder()
        .compute_derivative_group_linear(true);

    let temp0 = [*vk::DeviceQueueCreateInfo::builder().queue_priorities(&[1.])];

    let create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&temp0)
        .enabled_extension_names(&enabled_extension_names)
        .push_next(&mut features0)
        .push_next(&mut features1)
        .push_next(&mut features2);

    let device = unsafe { instance.create_device(*physical_device, &create_info, None) }.unwrap();

    // pSetLayouts[0]:                 const VkDescriptorSetLayout = 0x7e511920
    // pSetLayouts[1]:                 const VkDescriptorSetLayout = 0x7de00d60
    // pSetLayouts[2]:                 const VkDescriptorSetLayout = 0x7f5930001430
    // pSetLayouts[3]:                 const VkDescriptorSetLayout = 0x7f5930189b70

    let template0 = vk::ShaderStageFlags::empty()
        | vk::ShaderStageFlags::VERTEX
        | vk::ShaderStageFlags::TESSELLATION_CONTROL
        | vk::ShaderStageFlags::TESSELLATION_EVALUATION
        | vk::ShaderStageFlags::GEOMETRY
        | vk::ShaderStageFlags::FRAGMENT
        | vk::ShaderStageFlags::COMPUTE
        | vk::ShaderStageFlags::ALL
        | vk::ShaderStageFlags::RAYGEN_KHR
        | vk::ShaderStageFlags::ANY_HIT_KHR
        | vk::ShaderStageFlags::CLOSEST_HIT_KHR
        | vk::ShaderStageFlags::MISS_KHR
        | vk::ShaderStageFlags::INTERSECTION_KHR
        | vk::ShaderStageFlags::CALLABLE_KHR
        | vk::ShaderStageFlags::TASK_NV
        | vk::ShaderStageFlags::MESH_NV
        | vk::ShaderStageFlags::SUBPASS_SHADING_HUAWEI
        | vk::ShaderStageFlags::empty();

    let set_layouts = [
        unsafe { device.create_descriptor_set_layout( &*{
                   #[rustfmt::skip]
                    r#"
                        flags:                          VkDescriptorSetLayoutCreateFlags = 2 (VK_DESCRIPTOR_SET_LAYOUT_CREATE_UPDATE_AFTER_BIND_POOL_BIT)
        bindingCount:                   uint32_t = 1
        pBindings:                      const VkDescriptorSetLayoutBinding* = 0x11e6c0
            pBindings[0]:                   const VkDescriptorSetLayoutBinding = 0x11e6c0:
                binding:                        uint32_t = 0
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 2048
                stageFlags:                     VkShaderStageFlags = 2147483647 template0 
                pImmutableSamplers:             const VkSampler* = NULL
        pNext:                          VkDescriptorSetLayoutBindingFlagsCreateInfo = 0x11e560:
            sType:                          VkStructureType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO (1000161000)
            pNext:                          const void* = NULL
            bindingCount:                   uint32_t = 1
            pBindingFlags:                  const VkDescriptorBindingFlags* = 0x11e600
                pBindingFlags[0]:               const VkDescriptorBindingFlags = 15 (VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT 
 VK_DESCRIPTOR_BINDING_UPDATE_UNUSED_WHILE_PENDING_BIT 
 VK_DESCRIPTOR_BINDING_PARTIALLY_BOUND_BIT 
 VK_DESCRIPTOR_BINDING_VARIABLE_DESCRIPTOR_COUNT_BIT)
                "#;
                vk::DescriptorSetLayoutCreateInfo::builder()
                    .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
                    .bindings(&[*vk::DescriptorSetLayoutBinding::builder()
                        .descriptor_count(2048).descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(
                        template0)])
                    .push_next(
                        &mut vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
                            .binding_flags(&[vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING
                                | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT]),
                    )},
                None,
            )
        }
        .unwrap(),
        unsafe { device.create_descriptor_set_layout(
                &*{
                   #[rustfmt::skip]
            r#"        flags:                          VkDescriptorSetLayoutCreateFlags = 2 (VK_DESCRIPTOR_SET_LAYOUT_CREATE_UPDATE_AFTER_BIND_POOL_BIT)
        bindingCount:                   uint32_t = 3
        pBindings:                      const VkDescriptorSetLayoutBinding* = 0x11e6c0
            pBindings[0]:                   const VkDescriptorSetLayoutBinding = 0x11e6c0:
                binding:                        uint32_t = 0
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER (7)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = UNUSED
            pBindings[1]:                   const VkDescriptorSetLayoutBinding = 0x11e6d8:
                binding:                        uint32_t = 1
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER (7)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = UNUSED
            pBindings[2]:                   const VkDescriptorSetLayoutBinding = 0x11e6f0:
                binding:                        uint32_t = 2
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_MUTABLE_VALVE (1000351000)
                descriptorCount:                uint32_t = 1000000
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = UNUSED
        pNext:                          VkDescriptorSetLayoutBindingFlagsCreateInfo = 0x11e560:
            sType:                          VkStructureType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO (1000161000)
            pNext:                          const void* = VkMutableDescriptorTypeCreateInfoVALVE
            bindingCount:                   uint32_t = 3
            pBindingFlags:                  const VkDescriptorBindingFlags* = 0x11e600
                pBindingFlags[0]:               const VkDescriptorBindingFlags = 0
                pBindingFlags[1]:               const VkDescriptorBindingFlags = 0
                pBindingFlags[2]:               const VkDescriptorBindingFlags = 15 (VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT 
 VK_DESCRIPTOR_BINDING_UPDATE_UNUSED_WHILE_PENDING_BIT 
 VK_DESCRIPTOR_BINDING_PARTIALLY_BOUND_BIT 
 VK_DESCRIPTOR_BINDING_VARIABLE_DESCRIPTOR_COUNT_BIT)
        pNext:                          VkMutableDescriptorTypeCreateInfoVALVE = 0x11e5e0:
            sType:                          VkStructureType = VK_STRUCTURE_TYPE_MUTABLE_DESCRIPTOR_TYPE_CREATE_INFO_VALVE (1000351002)
            pNext:                          const void* = NULL
            mutableDescriptorTypeListCount: uint32_t = 3
            pMutableDescriptorTypeLists:    const VkMutableDescriptorTypeListVALVE* = 0x11e630
                pMutableDescriptorTypeLists[0]: const VkMutableDescriptorTypeListVALVE = 0x11e630:
                    descriptorTypeCount:            uint32_t = 0
                    pDescriptorTypes:               const VkDescriptorType* = NULL
                pMutableDescriptorTypeLists[1]: const VkMutableDescriptorTypeListVALVE = 0x11e640:
                    descriptorTypeCount:            uint32_t = 0
                    pDescriptorTypes:               const VkDescriptorType* = NULL
                pMutableDescriptorTypeLists[2]: const VkMutableDescriptorTypeListVALVE = 0x11e650:
                    descriptorTypeCount:            uint32_t = 5
                    pDescriptorTypes:               const VkDescriptorType* = 0x11e540
                        pDescriptorTypes[0]:            const VkDescriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER (7)
                        pDescriptorTypes[1]:            const VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE (2)
                        pDescriptorTypes[2]:            const VkDescriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER (4)
                        pDescriptorTypes[3]:            const VkDescriptorType = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE (3)
                        pDescriptorTypes[4]:            const VkDescriptorType = VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER (5)
"#
                    ;
                    vk::DescriptorSetLayoutCreateInfo::builder()
                    .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
                    .bindings(&[*vk::DescriptorSetLayoutBinding::builder()
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .stage_flags(
template0                        ),*vk::DescriptorSetLayoutBinding::builder().binding(1)
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .stage_flags(
template0                        ),*vk::DescriptorSetLayoutBinding::builder().binding(2)
                        .descriptor_count(1000000).descriptor_type(vk::DescriptorType::MUTABLE_VALVE)
                        .stage_flags(
template0                        )])
                    .push_next(
                        &mut vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
                            .binding_flags(&[vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING
                                | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT]),
                    )}
            , None) }.unwrap(),
        unsafe { device.create_descriptor_set_layout(
                &*{
                   #[rustfmt::skip]
            r#" 
        flags:                          VkDescriptorSetLayoutCreateFlags = 0
        bindingCount:                   uint32_t = 10
        pBindings:                      const VkDescriptorSetLayoutBinding* = 0xd25de0
            pBindings[0]:                   const VkDescriptorSetLayoutBinding = 0xd25de0:
                binding:                        uint32_t = 0
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb4fd0
                    pImmutableSamplers[0]:          const VkSampler = 0x7f5930000b70
            pBindings[1]:                   const VkDescriptorSetLayoutBinding = 0xd25df8:
                binding:                        uint32_t = 1
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb4fd8
                    pImmutableSamplers[0]:          const VkSampler = 0x7f5930000c50
            pBindings[2]:                   const VkDescriptorSetLayoutBinding = 0xd25e10:
                binding:                        uint32_t = 2
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb4fe0
                    pImmutableSamplers[0]:          const VkSampler = 0x7f5930000d30
            pBindings[3]:                   const VkDescriptorSetLayoutBinding = 0xd25e28:
                binding:                        uint32_t = 3
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb4fe8
                    pImmutableSamplers[0]:          const VkSampler = 0x7f5930000e10
            pBindings[4]:                   const VkDescriptorSetLayoutBinding = 0xd25e40:
                binding:                        uint32_t = 4
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb4ff0
                    pImmutableSamplers[0]:          const VkSampler = 0x7f5930000ef0
            pBindings[5]:                   const VkDescriptorSetLayoutBinding = 0xd25e58:
                binding:                        uint32_t = 5
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb4ff8
                    pImmutableSamplers[0]:          const VkSampler = 0x7f5930000fd0
            pBindings[6]:                   const VkDescriptorSetLayoutBinding = 0xd25e70:
                binding:                        uint32_t = 6
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb5000
                    pImmutableSamplers[0]:          const VkSampler = 0x7f59300010b0
            pBindings[7]:                   const VkDescriptorSetLayoutBinding = 0xd25e88:
                binding:                        uint32_t = 7
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb5008
                    pImmutableSamplers[0]:          const VkSampler = 0x7f5930001190
            pBindings[8]:                   const VkDescriptorSetLayoutBinding = 0xd25ea0:
                binding:                        uint32_t = 8
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb5010
                    pImmutableSamplers[0]:          const VkSampler = 0x7f5930001270
            pBindings[9]:                   const VkDescriptorSetLayoutBinding = 0xd25eb8:
                binding:                        uint32_t = 9
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER (0)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = 0xcb5018
                    pImmutableSamplers[0]:          const VkSampler = 0x7f5930001350


            "#
                    ;
                    vk::DescriptorSetLayoutCreateInfo::builder()
                    .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
                    .bindings(&[*vk::DescriptorSetLayoutBinding::builder().binding(1)
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(
template0                        ),*vk::DescriptorSetLayoutBinding::builder().binding(1)
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(
template0                        ),*vk::DescriptorSetLayoutBinding::builder().binding(1)
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(
template0                        ),*vk::DescriptorSetLayoutBinding::builder().binding(1)
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(
template0                        ),*vk::DescriptorSetLayoutBinding::builder().binding(1)
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(
template0                        ),*vk::DescriptorSetLayoutBinding::builder().binding(1)
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(
template0                        ),*vk::DescriptorSetLayoutBinding::builder().binding(1)
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(
template0                        ),*vk::DescriptorSetLayoutBinding::builder().binding(1)
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::SAMPLER)
                        .stage_flags(
template0                        ),])
                    .push_next(
                        &mut vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
                            .binding_flags(&[vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING
                                | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT]),
                    )}
            , None) }.unwrap(),
        unsafe { device.create_descriptor_set_layout(
                &*{
                   #[rustfmt::skip]
            r#"  
        flags:                          VkDescriptorSetLayoutCreateFlags = 1 (VK_DESCRIPTOR_SET_LAYOUT_CREATE_PUSH_DESCRIPTOR_BIT_KHR)
        bindingCount:                   uint32_t = 1
        pBindings:                      const VkDescriptorSetLayoutBinding* = 0xa6c410
            pBindings[0]:                   const VkDescriptorSetLayoutBinding = 0xa6c410:
                binding:                        uint32_t = 0
                descriptorType:                 VkDescriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER (6)
                descriptorCount:                uint32_t = 1
                stageFlags:                     VkShaderStageFlags = 2147483647 template0
                pImmutableSamplers:             const VkSampler* = UNUSED
            "#
                    ;
                    vk::DescriptorSetLayoutCreateInfo::builder()
                    .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
                    .bindings(&[*vk::DescriptorSetLayoutBinding::builder()
                        .descriptor_count(1).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .stage_flags(
                            template0
                        )]
                        )}
            , None) }.unwrap(),
    ];

    let pipeline_layout = unsafe {
        device.create_pipeline_layout(
            &vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layouts),
            None,
        )
    }
    .unwrap();

    let mut rng = rand::thread_rng();
    let mut file = std::fs::File::open(match rng.gen_range(0..=2) {
        0 => "data/150-0.bin",
        1 => "data/151-0.bin",
        2 => "data/152-0.bin",
        _ => unreachable!(),
    })
    .unwrap();

    // Read the contents of the file into a Vec<u8>
    let mut code_vec = Vec::new();
    file.read_to_end(&mut code_vec).unwrap();

    let (_, code) = nom::multi::many0(nom::number::complete::u32::<_, nom::error::Error<&[u8]>>(
        nom::number::Endianness::Native,
    ))
    .parse(code_vec.as_slice())
    .unwrap();

    let create_info = vk::ShaderModuleCreateInfo::builder().code(&code);

    let shader_module = unsafe { device.create_shader_module(&create_info, None) }.unwrap();
    let _ = code_vec;

    let name = CString::new("main").unwrap();
    let create_info = vk::ComputePipelineCreateInfo::builder()
        .stage(
            *vk::PipelineShaderStageCreateInfo::builder()
                .flags(vk::PipelineShaderStageCreateFlags::REQUIRE_FULL_SUBGROUPS)
                .stage(vk::ShaderStageFlags::COMPUTE)
                .module(shader_module)
                .name(name.as_c_str()),
        )
        .layout(pipeline_layout);
    let _ = name;

    let _ = dbg!(unsafe {
        device.create_compute_pipelines(vk::PipelineCache::null(), &[*create_info], None)
    });
}
