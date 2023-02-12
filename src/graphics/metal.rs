use crate::native::apple::{
    apple_util::{self, msg_send_},
    frameworks::*,
};

use super::*;

// https://developer.apple.com/metal/Metal-Feature-Set-Tables.pdf
const MAX_BUFFERS_PER_STAGE: usize = 14;
const MAX_UNIFORM_BUFFER_SIZE: u64 = 4 * 1024 * 1024;
const MAX_VERTEX_ATTRIBUTES: usize = 31;
const UNIFORM_BUFFER_INDEX: u64 = 0;
const NUM_INFLIGHT_FRAMES: usize = 3;
#[cfg(any(target_os = "macos", all(target_os = "ios", target_arch = "x86_64")))]
const UNIFORM_BUFFER_ALIGN: u64 = 256;
#[cfg(all(target_os = "ios", not(target_arch = "x86_64")))]
const UNIFORM_BUFFER_ALIGN: u64 = 16;

impl From<VertexFormat> for MTLVertexFormat {
    fn from(vf: VertexFormat) -> Self {
        match vf {
            VertexFormat::Float1 => MTLVertexFormat::Float,
            VertexFormat::Float2 => MTLVertexFormat::Float2,
            VertexFormat::Float3 => MTLVertexFormat::Float3,
            VertexFormat::Float4 => MTLVertexFormat::Float4,
            VertexFormat::Byte1 => MTLVertexFormat::UChar,
            VertexFormat::Byte2 => MTLVertexFormat::UChar2,
            VertexFormat::Byte3 => MTLVertexFormat::UChar3,
            VertexFormat::Byte4 => MTLVertexFormat::UChar4,
            VertexFormat::Short1 => MTLVertexFormat::Short,
            VertexFormat::Short2 => MTLVertexFormat::Short2,
            VertexFormat::Short3 => MTLVertexFormat::Short3,
            VertexFormat::Short4 => MTLVertexFormat::Short4,
            VertexFormat::Int1 => MTLVertexFormat::Int,
            VertexFormat::Int2 => MTLVertexFormat::Int2,
            VertexFormat::Int3 => MTLVertexFormat::Int3,
            VertexFormat::Int4 => MTLVertexFormat::Int4,
            VertexFormat::Mat4 => MTLVertexFormat::Float4,
            _ => unreachable!(),
        }
    }
}

impl From<UniformType> for MTLVertexFormat {
    fn from(ut: UniformType) -> Self {
        match ut {
            UniformType::Float1 => MTLVertexFormat::Float,
            UniformType::Float2 => MTLVertexFormat::Float2,
            UniformType::Float3 => MTLVertexFormat::Float3,
            UniformType::Float4 => MTLVertexFormat::Float4,
            UniformType::Int1 => MTLVertexFormat::Int,
            UniformType::Int2 => MTLVertexFormat::Int2,
            UniformType::Int3 => MTLVertexFormat::Int3,
            UniformType::Int4 => MTLVertexFormat::Int4,
            UniformType::Mat4 => MTLVertexFormat::Float4,
            _ => unreachable!(),
        }
    }
}

impl From<Comparison> for MTLCompareFunction {
    fn from(cmp: Comparison) -> Self {
        match cmp {
            Comparison::Never => MTLCompareFunction::Never,
            Comparison::Less => MTLCompareFunction::Less,
            Comparison::LessOrEqual => MTLCompareFunction::LessEqual,
            Comparison::Greater => MTLCompareFunction::Greater,
            Comparison::GreaterOrEqual => MTLCompareFunction::GreaterEqual,
            Comparison::Equal => MTLCompareFunction::Equal,
            Comparison::NotEqual => MTLCompareFunction::NotEqual,
            Comparison::Always => MTLCompareFunction::Always,
        }
    }
}

impl From<BlendFactor> for MTLBlendFactor {
    fn from(factor: BlendFactor) -> Self {
        match factor {
            BlendFactor::Zero => MTLBlendFactor::Zero,
            BlendFactor::One => MTLBlendFactor::One,
            BlendFactor::Value(BlendValue::SourceColor) => MTLBlendFactor::SourceColor,
            BlendFactor::Value(BlendValue::SourceAlpha) => MTLBlendFactor::SourceAlpha,
            BlendFactor::Value(BlendValue::DestinationColor) => MTLBlendFactor::DestinationColor,
            BlendFactor::Value(BlendValue::DestinationAlpha) => MTLBlendFactor::DestinationAlpha,
            BlendFactor::OneMinusValue(BlendValue::SourceColor) => {
                MTLBlendFactor::OneMinusSourceColor
            }
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => {
                MTLBlendFactor::OneMinusSourceAlpha
            }
            BlendFactor::OneMinusValue(BlendValue::DestinationColor) => {
                MTLBlendFactor::OneMinusDestinationColor
            }
            BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => {
                MTLBlendFactor::OneMinusDestinationAlpha
            }
            BlendFactor::SourceAlphaSaturate => MTLBlendFactor::SourceAlphaSaturated,
        }
    }
}

// impl From<StencilOp> for MTLStencilOperation {
//     fn from(op: StencilOp) -> Self {
//         match op {
//             StencilOp::Keep => MTLStencilOperation::Keep,
//             StencilOp::Zero => MTLStencilOperation::Zero,
//             StencilOp::Replace => MTLStencilOperation::Replace,
//             StencilOp::IncrementClamp => MTLStencilOperation::IncrementClamp,
//             StencilOp::DecrementClamp => MTLStencilOperation::DecrementClamp,
//             StencilOp::Invert => MTLStencilOperation::Invert,
//             StencilOp::IncrementWrap => MTLStencilOperation::IncrementWrap,
//             StencilOp::DecrementWrap => MTLStencilOperation::DecrementWrap,
//         }
//     }
// }

impl From<CompareFunc> for MTLCompareFunction {
    fn from(cf: CompareFunc) -> Self {
        match cf {
            CompareFunc::Always => MTLCompareFunction::Always,
            CompareFunc::Never => MTLCompareFunction::Never,
            CompareFunc::Less => MTLCompareFunction::Less,
            CompareFunc::Equal => MTLCompareFunction::Equal,
            CompareFunc::LessOrEqual => MTLCompareFunction::LessEqual,
            CompareFunc::Greater => MTLCompareFunction::Greater,
            CompareFunc::NotEqual => MTLCompareFunction::NotEqual,
            CompareFunc::GreaterOrEqual => MTLCompareFunction::GreaterEqual,
        }
    }
}

impl From<VertexStep> for MTLVertexStepFunction {
    fn from(vs: VertexStep) -> Self {
        match vs {
            VertexStep::PerVertex => MTLVertexStepFunction::PerVertex,
            VertexStep::PerInstance => MTLVertexStepFunction::PerInstance,
        }
    }
}

impl From<PrimitiveType> for MTLPrimitiveType {
    fn from(primitive_type: PrimitiveType) -> Self {
        match primitive_type {
            PrimitiveType::Triangles => MTLPrimitiveType::Triangle,
            PrimitiveType::Lines => MTLPrimitiveType::Line,
        }
    }
}

impl Usage {
    fn to_u64(self) -> u64 {
        match self {
            Usage::Immutable => MTLResourceOptions::StorageModeShared,
            #[cfg(target_os = "macos")]
            Usage::Dynamic | Usage::Stream => {
                MTLResourceOptions::CPUCacheModeWriteCombined
                    | MTLResourceOptions::StorageModeManaged
            }
            #[cfg(target_os = "ios")]
            Usage::Dynamic | Usage::Stream => MTLResourceOptions::CPUCacheModeWriteCombined,
        }
    }
}

impl From<TextureFormat> for MTLPixelFormat {
    fn from(format: TextureFormat) -> Self {
        match format {
            TextureFormat::RGBA8 => MTLPixelFormat::RGBA8Unorm,
            //TODO: Depth16Unorm ?
            TextureFormat::Depth => MTLPixelFormat::Depth32Float_Stencil8,
            _ => todo!(),
        }
    }
}

// impl From<CullFace> for MTLCullMode {
//     fn from(cull_face: CullFace) -> Self {
//         match cull_face {
//             CullFace::Back => MTLCullMode::Back,
//             CullFace::Front => MTLCullMode::Front,
//             CullFace::Nothing => MTLCullMode::None,
//         }
//     }
// }

// impl From<FrontFaceOrder> for MTLWinding {
//     fn from(order: FrontFaceOrder) -> Self {
//         match order {
//             FrontFaceOrder::Clockwise => MTLWinding::Clockwise,
//             FrontFaceOrder::CounterClockwise => MTLWinding::CounterClockwise,
//         }
//     }
// }

#[inline]
fn roundup_ub_buffer(current_buffer: u64) -> u64 {
    ((current_buffer) + ((UNIFORM_BUFFER_ALIGN) - 1)) & !((UNIFORM_BUFFER_ALIGN) - 1)
}

const WTF: usize = 200;
#[derive(Clone, Copy, Debug)]
pub struct Buffer {
    raw: [ObjcId; WTF],
    buffer_type: BufferType,
    size: usize,
    index_type: Option<IndexType>,
    value: usize,
    next_value: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ShaderUniform {
    size: u64,
    offset: u64,
    format: MTLVertexFormat,
}

#[derive(Debug)]
struct ShaderInternal {
    vertex_function: ObjcId,
    fragment_function: ObjcId,
    uniforms: Vec<ShaderUniform>,
    // the distance, in bytes, between two uniforms in uniforms buffer
    stride: u64,
}

struct RenderPassInternal {
    render_pass_desc: ObjcId,
    texture: TextureId,
    depth_texture: Option<TextureId>,
}

#[derive(Clone, Debug)]
struct PipelineInternal {
    pipeline_state: ObjcId,
    depth_stencil_state: ObjcId,
    layout: Vec<BufferLayout>,
    //attributes: Vec<VertexAttributeInternal>,
    shader: ShaderId,
    params: PipelineParams,
}

struct TextureInternal {
    texture: ObjcId,
    sampler: ObjcId,
    width: u32,
    height: u32,
}

pub struct MetalContext {
    buffers: Vec<Buffer>,
    shaders: Vec<ShaderInternal>,
    pipelines: Vec<PipelineInternal>,
    textures: Vec<TextureInternal>,
    passes: Vec<RenderPassInternal>,
    command_queue: ObjcId,
    command_buffer: Option<ObjcId>,
    render_encoder: Option<ObjcId>,
    view: ObjcId,
    device: ObjcId,
    current_frame_index: usize,
    uniform_buffers: [ObjcId; 3],
    // cached index_buffer from apply_bindings
    index_buffer: Option<ObjcId>,
    // cached pipeline from apply_pipeline
    current_pipeline: Option<Pipeline>,
    current_ub_offset: u64,
}

impl MetalContext {
    pub fn new() -> MetalContext {
        unsafe {
            let view = crate::window::apple_view().unwrap();
            assert!(!view.is_null());
            let device: ObjcId = msg_send![view, device];
            assert!(!device.is_null());
            let command_queue: ObjcId = msg_send![device, newCommandQueue];

            if false {
                let capture_manager = msg_send_![class![MTLCaptureManager], sharedCaptureManager];
                assert!(!capture_manager.is_null());

                let MTLCaptureDestinationGPUTraceDocument = 2u64;
                if !msg_send![
                    capture_manager,
                    supportsDestination: MTLCaptureDestinationGPUTraceDocument
                ] {
                    panic!("capture failed");
                }

                let capture_descriptor =
                    msg_send_![msg_send_![class![MTLCaptureDescriptor], alloc], init];
                msg_send_![capture_descriptor, setCaptureObject: device];
                msg_send_![
                    capture_descriptor,
                    setDestination: MTLCaptureDestinationGPUTraceDocument
                ];
                let path = apple_util::str_to_nsstring("/Users/fedor/wtf1.gputrace");
                let url = msg_send_![class!(NSURL), fileURLWithPath: path];
                msg_send_![capture_descriptor, setOutputURL: url];

                let mut error: ObjcId = nil;
                if !msg_send![capture_manager, startCaptureWithDescriptor:capture_descriptor
                              error:&mut error]
                {
                    let description: ObjcId = msg_send![error, localizedDescription];
                    let string = apple_util::nsstring_to_string(description);
                    panic!("Capture error: {}", string);
                }
            }

            let uniform_buffers = [
                msg_send![device, newBufferWithLength:MAX_UNIFORM_BUFFER_SIZE
                          options:Usage::Stream.to_u64()],
                msg_send![device, newBufferWithLength:MAX_UNIFORM_BUFFER_SIZE
                          options:Usage::Stream.to_u64()],
                msg_send![device, newBufferWithLength:MAX_UNIFORM_BUFFER_SIZE
                          options:Usage::Stream.to_u64()],
            ];

            MetalContext {
                command_queue,
                command_buffer: None,
                render_encoder: None,
                view,
                device,
                buffers: vec![],
                shaders: vec![],
                pipelines: vec![],
                textures: vec![],
                passes: vec![],
                index_buffer: None,
                current_pipeline: None,
                uniform_buffers,
                current_frame_index: 1,
                current_ub_offset: 0,
            }
        }
    }
}

impl RenderingBackend for MetalContext {
    fn delete_render_pass(&mut self, render_pass: RenderPass) {}
    fn pipeline_set_blend(&mut self, pipeline: &Pipeline, color_blend: Option<BlendState>) {}
    fn new_buffer_index_stream(&mut self, index_type: IndexType, size: usize) -> BufferId {
        unimplemented!()
    }
    fn buffer_size(&mut self, buffer: BufferId) -> usize {
        unimplemented!()
    }
    fn buffer_delete(&mut self, buffer: BufferId) {}
    fn set_cull_face(&mut self, cull_face: CullFace) {}
    fn set_color_write(&mut self, color_write: ColorMask) {}
    fn set_blend(&mut self, color_blend: Option<BlendState>, alpha_blend: Option<BlendState>) {}
    fn set_stencil(&mut self, stencil_test: Option<StencilState>) {}
    fn apply_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {}
    fn apply_scissor_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {}
    fn texture_set_filter(&self, texture: TextureId, filter: FilterMode) {}
    fn texture_set_wrap(&mut self, texture: TextureId, wrap: TextureWrap) {}
    fn texture_resize(&mut self, texture: TextureId, width: u32, height: u32, bytes: Option<&[u8]>) {}
    fn texture_read_pixels(&mut self, texture: TextureId, bytes: &mut [u8]) {}
    fn clear(&self, color: Option<(f32, f32, f32, f32)>, depth: Option<f32>, stencil: Option<i32>) {
    }

    fn new_render_pass(
        &mut self,
        color_img: TextureId,
        depth_img: Option<TextureId>,
    ) -> RenderPass {
        unsafe {
            let render_pass_desc =
                msg_send_![class!(MTLRenderPassDescriptor), renderPassDescriptor];
            msg_send_![render_pass_desc, retain];
            assert!(!render_pass_desc.is_null());
            let color_texture = self.textures[color_img.0].texture;
            let color_attachment = msg_send_![msg_send_![render_pass_desc, colorAttachments], objectAtIndexedSubscript:0];
            msg_send_![color_attachment, setTexture: color_texture];
            msg_send_![color_attachment, setLoadAction: MTLLoadAction::Clear];
            msg_send_![color_attachment, setStoreAction: MTLStoreAction::Store];

            if let Some(depth_img) = depth_img {
                let depth_texture = self.textures[depth_img.0].texture;

                let depth_attachment = msg_send_![render_pass_desc, depthAttachment];
                msg_send_![depth_attachment, setTexture: depth_texture];
                msg_send_![depth_attachment, setLoadAction: MTLLoadAction::Clear];
                msg_send_![depth_attachment, setStoreAction: MTLStoreAction::Store];
                msg_send_![depth_attachment, setClearDepth:1.];

                let stencil_attachment = msg_send_![render_pass_desc, stencilAttachment];
                msg_send_![stencil_attachment, setTexture: depth_texture];
            }
            let pass = RenderPassInternal {
                render_pass_desc,
                texture: color_img,
                depth_texture: depth_img,
            };

            self.passes.push(pass);

            RenderPass(self.passes.len() - 1)
        }
    }
    fn render_pass_texture(&self, render_pass: RenderPass) -> TextureId {
        self.passes[render_pass.0].texture
    }
    fn new_buffer_immutable(&mut self, buffer_type: BufferType, data: BufferSource) -> BufferId {
        debug_assert!(data.is_slice);
        let index_type = if buffer_type == BufferType::IndexBuffer {
            Some(IndexType::for_type_size(data.element_size))
        } else {
            None
        };

        let size = data.size as u64;
        let mut raw = [nil; WTF];
        for i in 0..WTF {
            let buffer: ObjcId = unsafe {
                msg_send![self.device,
                      newBufferWithBytes:data.ptr
                      length:size
                      options:MTLResourceOptions::StorageModeShared]
            };
            unsafe {
                msg_send_![buffer, retain];
            }
            raw[i] = buffer;
        }
        let buffer = Buffer {
            raw,
            buffer_type,
            size: size as usize,
            index_type,
            value: 0,
            next_value: 0,
        };
        self.buffers.push(buffer);
        BufferId(self.buffers.len() - 1)
    }

    fn new_buffer_stream(&mut self, buffer_type: BufferType, size: usize) -> BufferId {
        let mut raw = [nil; WTF];
        for i in 0..WTF {
            let buffer: ObjcId = unsafe {
                msg_send![self.device,
                      newBufferWithLength:size
                      options:MTLResourceOptions::CPUCacheModeWriteCombined | MTLResourceOptions::StorageModeManaged]
            };
            unsafe {
                msg_send_![buffer, retain];
            }
            raw[i] = buffer;
        }
        let buffer = Buffer {
            raw,
            buffer_type,
            size: size as usize,
            index_type: None,
            value: 0,
            next_value: 0,
        };
        self.buffers.push(buffer);
        BufferId(self.buffers.len() - 1)
    }

    fn buffer_update(&mut self, buffer: BufferId, data: BufferSource) {
        let mut buffer = &mut self.buffers[buffer.0];
        assert!(data.size <= buffer.size);

        unsafe {
            let dest: *mut std::ffi::c_void = msg_send![buffer.raw[buffer.next_value], contents];
            std::ptr::copy(data.ptr, dest, data.size);

            #[cfg(target_os = "macos")]
            msg_send_![buffer.raw[buffer.next_value], didModifyRange:NSRange::new(0, data.size as u64)];
        }
        buffer.value = buffer.next_value;
    }

    fn new_shader(
        &mut self,
        shader: ShaderSource,
        meta: ShaderMeta,
    ) -> Result<ShaderId, ShaderError> {
        unsafe {
            let shader = apple_util::str_to_nsstring(shader.metal_shader.unwrap());
            let mut error: ObjcId = nil;
            let library: ObjcId = msg_send![
                self.device,
                newLibraryWithSource: shader
                options:nil
                error: &mut error
            ];
            if library.is_null() {
                let description: ObjcId = msg_send![error, localizedDescription];
                let string = apple_util::nsstring_to_string(description);
                panic!("Shader {}", string);
            }

            let vertex_function: ObjcId = msg_send![library, newFunctionWithName: apple_util::str_to_nsstring("vertexShader")];
            assert!(!vertex_function.is_null());
            let fragment_function: ObjcId = msg_send![library, newFunctionWithName: apple_util::str_to_nsstring("fragmentShader")];
            assert!(!fragment_function.is_null());

            let mut stride = 0;
            let mut index = 0;
            let uniforms = meta
                .uniforms
                .uniforms
                .iter()
                .scan(0, |offset, uniform| {
                    let size = uniform.uniform_type.size() as u64;
                    stride += size;
                    let shader_uniform = ShaderUniform {
                        size: uniform.uniform_type.size() as u64,
                        offset: *offset,
                        format: uniform.uniform_type.into(),
                    };
                    index += 1;
                    *offset += size as u64;
                    Some(shader_uniform)
                })
                .collect();

            let shader = ShaderInternal {
                vertex_function,
                fragment_function,
                uniforms,
                stride,
            };
            self.shaders.push(shader);
            Ok(ShaderId(self.shaders.len() - 1))
        }
    }

    fn new_texture(
        &mut self,
        access: TextureAccess,
        bytes: Option<&[u8]>,
        params: TextureParams,
    ) -> TextureId {
        let descriptor = unsafe { msg_send_![class!(MTLTextureDescriptor), new] };
        unsafe {
            msg_send_![descriptor, retain];
        }
        unsafe {
            msg_send_![descriptor, setWidth:params.width as u64];
            msg_send_![descriptor, setHeight:params.height as u64];
            msg_send_![descriptor, setCpuCacheMode: MTLCPUCacheMode::DefaultCache];
            msg_send_![descriptor, setPixelFormat: MTLPixelFormat::from(params.format)];

            if access == TextureAccess::RenderTarget {
                if params.format != TextureFormat::Depth {
                    msg_send_![descriptor, setPixelFormat: MTLPixelFormat::RGBA8Unorm];
                }
                msg_send_![descriptor, setStorageMode: MTLStorageMode::Private];
                msg_send_![
                    descriptor,
                    setUsage: MTLTextureUsage::RenderTarget as u64
                        | MTLTextureUsage::ShaderRead as u64
                        | MTLTextureUsage::ShaderWrite as u64
                ];
            } else {
                msg_send_![descriptor, setUsage: MTLTextureUsage::ShaderRead];
                #[cfg(target_os = "macos")]
                {
                    msg_send_![descriptor, setStorageMode: MTLStorageMode::Managed];
                    msg_send_![
                        descriptor,
                        setResourceOptions: MTLResourceOptions::StorageModeManaged
                    ];
                }
                // #[cfg(target_os = "ios")]
                // {
                //     texture_dsc.set_storage_mode(MTLStorageMode::Shared);
                //     texture_dsc.set_resource_options(MTLResourceOptions::StorageModeShared);
                // }
            }
        };

        let texture = unsafe {
            let sampler_dsc = msg_send_![class!(MTLSamplerDescriptor), new];
            unsafe {
                msg_send_![sampler_dsc, retain];
            }
            msg_send_![sampler_dsc, setMinFilter: MTLSamplerMinMagFilter::Linear];
            msg_send_![sampler_dsc, setMagFilter: MTLSamplerMinMagFilter::Linear];

            let sampler_state = msg_send_![self.device, newSamplerStateWithDescriptor: sampler_dsc];
            unsafe {
                msg_send_![sampler_state, retain];
            }
            let raw_texture = msg_send_![self.device, newTextureWithDescriptor: descriptor];

            unsafe {
                msg_send_![raw_texture, retain];
            }
            self.textures.push(TextureInternal {
                sampler: sampler_state,
                texture: raw_texture,
                width: params.width as _,
                height: params.height as _,
            });
            TextureId(self.textures.len() - 1)
        };

        if let Some(bytes) = bytes {
            assert_eq!(
                params.format.size(params.width, params.height) as usize,
                bytes.len()
            );

            self.texture_update_part(texture, 0, 0, params.width as _, params.height as _, bytes);
        }
        texture
    }

    fn texture_update_part(
        &mut self,
        texture: TextureId,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        bytes: &[u8],
    ) {
        let raw_texture = self.textures[texture.0].texture;
        let region = MTLRegion {
            origin: MTLOrigin {
                x: x_offset as u64,
                y: y_offset as u64,
                z: 0,
            },
            size: MTLSize {
                width: width as u64,
                height: height as u64,
                depth: 1,
            },
        };
        unsafe {
            msg_send_![raw_texture, replaceRegion:region
                       mipmapLevel:0
                       withBytes:bytes.as_ptr()
                       bytesPerRow:(width * 4) as u64];
        }
    }

    fn texture_size(&self, texture: TextureId) -> (u32, u32) {
        let texture = &self.textures[texture.0];
        (texture.width as _, texture.height as _)
    }

    fn new_pipeline(
        &mut self,
        buffer_layout: &[BufferLayout],
        attributes: &[VertexAttribute],
        shader: ShaderId,
    ) -> Pipeline {
        self.new_pipeline_with_params(buffer_layout, attributes, shader, Default::default())
    }

    fn new_pipeline_with_params(
        &mut self,
        buffer_layout: &[BufferLayout],
        attributes: &[VertexAttribute],
        shader: ShaderId,
        params: PipelineParams,
    ) -> Pipeline {
        unsafe {
            let shader_internal = &self.shaders[shader.0];

            let vertex_descriptor: ObjcId =
                msg_send![class!(MTLVertexDescriptor), vertexDescriptor];

            let attribute = |i, buffer_index, offset, format: MTLVertexFormat| {
                let mtl_attribute_desc = msg_send_![
                    msg_send_![vertex_descriptor, attributes],
                    objectAtIndexedSubscript: i
                ];
                msg_send_![mtl_attribute_desc, setBufferIndex: buffer_index];
                msg_send_![mtl_attribute_desc, setOffset: offset];
                msg_send_![mtl_attribute_desc, setFormat: format];
            };
            let layout = |i, step_func: VertexStep, stride, step_rate| {
                let mtl_buffer_desc = msg_send_![
                    msg_send_![vertex_descriptor, layouts],
                    objectAtIndexedSubscript: i
                ];
                let step_func: MTLVertexStepFunction = step_func.into();
                msg_send_![mtl_buffer_desc, setStride: stride];
                msg_send_![mtl_buffer_desc, setStepFunction: step_func];
                msg_send_![mtl_buffer_desc, setStepRate: step_rate];
            };

            let mut offsets = [0u64; 50];
            for (i, a) in attributes.iter().enumerate() {
                let offset = &mut offsets[a.buffer_index];
                attribute(
                    i as u64,
                    a.buffer_index as u64 + 1,
                    *offset,
                    a.format.into(),
                );
                *offset += a.format.size_bytes() as u64;
            }
            for (i, buffer) in buffer_layout.iter().enumerate() {
                layout(
                    i as u64 + 1,
                    buffer.step_func,
                    if buffer.stride == 0 {
                        offsets[i]
                    } else {
                        buffer.stride as u64
                    },
                    buffer.step_rate as u64,
                );
            }

            let descriptor = msg_send_![class!(MTLRenderPipelineDescriptor), new];
            msg_send_![descriptor, setVertexFunction:shader_internal.vertex_function];
            msg_send_![descriptor, setFragmentFunction:shader_internal.fragment_function];
            msg_send_![descriptor, setVertexDescriptor: vertex_descriptor];
            let color_attachments = msg_send_![descriptor, colorAttachments];
            let color_attachment = msg_send_![color_attachments, objectAtIndexedSubscript: 0];

            let view_pixel_format: MTLPixelFormat = msg_send![self.view, colorPixelFormat];
            msg_send_![color_attachment, setPixelFormat: view_pixel_format];
            if params.color_blend.is_some() {
                msg_send_![color_attachment, setBlendingEnabled: true];
                //TODO: Set from pipe params
                msg_send_![
                    color_attachment,
                    setRgbBlendOperation: MTLBlendOperation::Add
                ];
                msg_send_![
                    color_attachment,
                    setAlphaBlendOperation: MTLBlendOperation::Add
                ];
                msg_send_![
                    color_attachment,
                    setSourceRGBBlendFactor: MTLBlendFactor::SourceAlpha
                ];
                msg_send_![
                    color_attachment,
                    setSourceRGBBlendFactor: MTLBlendFactor::SourceAlpha
                ];
                msg_send_![
                    color_attachment,
                    setSourceAlphaBlendFactor: MTLBlendFactor::SourceAlpha
                ];
                msg_send_![
                    color_attachment,
                    setDestinationRGBBlendFactor: MTLBlendFactor::OneMinusSourceAlpha
                ];
                msg_send_![
                    color_attachment,
                    setDestinationAlphaBlendFactor: MTLBlendFactor::OneMinusSourceAlpha
                ];
            }

            msg_send_![
                descriptor,
                setDepthAttachmentPixelFormat: MTLPixelFormat::Depth32Float_Stencil8
            ];
            msg_send_![
                descriptor,
                setStencilAttachmentPixelFormat: MTLPixelFormat::Depth32Float_Stencil8
            ];

            let mut error: ObjcId = nil;
            let pipeline_state: ObjcId = msg_send![
                self.device,
                newRenderPipelineStateWithDescriptor: descriptor
                error: &mut error
            ];
            if pipeline_state.is_null() {
                let description: ObjcId = msg_send![error, localizedDescription];
                let string = apple_util::nsstring_to_string(description);
                panic!("newRenderPipelineStateWithDescriptor error: {}", string);
            }

            let depth_stencil_desc = msg_send_![class!(MTLDepthStencilDescriptor), new];
            msg_send_![depth_stencil_desc, setDepthWriteEnabled: BOOL::from(params.depth_write)];
            msg_send_![depth_stencil_desc, setDepthCompareFunction: MTLCompareFunction::from(params.depth_test)];

            // if let Some(stencil_test) = params.stencil_test {
            //     let back_face_stencil_desc = StencilDescriptor::new();
            //     back_face_stencil_desc.set_stencil_compare_function(stencil_test.back.test_func.into());
            //     back_face_stencil_desc.set_stencil_failure_operation(stencil_test.back.fail_op.into());
            //     back_face_stencil_desc
            //         .set_depth_failure_operation(stencil_test.back.depth_fail_op.into());
            //     back_face_stencil_desc.set_read_mask(stencil_test.back.test_mask);
            //     back_face_stencil_desc.set_write_mask(stencil_test.back.write_mask);

            //     depth_stencil_desc.set_back_face_stencil(Some(back_face_stencil_desc.as_ref()));

            //     let front_face_stencil_desc = StencilDescriptor::new();
            //     front_face_stencil_desc
            //         .set_stencil_compare_function(stencil_test.front.test_func.into());
            //     front_face_stencil_desc
            //         .set_stencil_failure_operation(stencil_test.front.fail_op.into());
            //     front_face_stencil_desc
            //         .set_depth_failure_operation(stencil_test.front.depth_fail_op.into());
            //     front_face_stencil_desc.set_read_mask(stencil_test.front.test_mask);
            //     front_face_stencil_desc.set_write_mask(stencil_test.front.write_mask);

            //     depth_stencil_desc.set_front_face_stencil(Some(front_face_stencil_desc.as_ref()))
            // }

            let depth_stencil_state = msg_send_![
                self.device,
                newDepthStencilStateWithDescriptor: depth_stencil_desc
            ];

            let pipeline = PipelineInternal {
                pipeline_state,
                depth_stencil_state,
                layout: buffer_layout.to_vec(),
                //attributes: vertex_layout,
                shader,
                params,
            };

            self.pipelines.push(pipeline);

            Pipeline(self.pipelines.len() - 1)
        }
    }

    fn apply_pipeline(&mut self, pipeline: &Pipeline) {
        assert!(
            self.render_encoder.is_some(),
            "apply_pipeline before begin_pass"
        );
        let render_encoder = self.render_encoder.unwrap();

        unsafe {
            self.current_pipeline = Some(*pipeline);
            let pipeline = &self.pipelines[pipeline.0];

            msg_send_![render_encoder, setRenderPipelineState: pipeline.pipeline_state];
            msg_send_![render_encoder, setDepthStencilState:pipeline.depth_stencil_state];
            // render_encoder.set_front_facing_winding(pipeline.params.front_face_order.into());
            // render_encoder.set_cull_mode(pipeline.params.cull_face.into());
        }
    }

    fn apply_bindings(&mut self, bindings: &Bindings) {
        assert!(
            self.render_encoder.is_some(),
            "apply_bindings before begin_pass"
        );

        unsafe {
            let render_encoder = self.render_encoder.unwrap();
            for (index, vertex_buffer) in bindings.vertex_buffers.iter().enumerate() {
                let buffer = &mut self.buffers[vertex_buffer.0];
                let () = msg_send![render_encoder,
                                   setVertexBuffer:buffer.raw[buffer.value]
                                   offset:0
                                   atIndex:(index + 1) as u64];
                buffer.next_value = buffer.value + 1;
            }
            let mut index_buffer = &mut self.buffers[bindings.index_buffer.0];
            self.index_buffer = Some(index_buffer.raw[index_buffer.value]);
            index_buffer.next_value = index_buffer.value + 1;

            let img_count = bindings.images.len();
            if img_count > 0 {
                for (n, img) in bindings.images.iter().enumerate() {
                    let TextureInternal {
                        sampler, texture, ..
                    } = self.textures[img.0];
                    msg_send_![render_encoder, setFragmentSamplerState:sampler
                               atIndex:n
                    ];
                    msg_send_![render_encoder, setFragmentTexture:texture
                               atIndex:n
                    ];
                }
            }
        }
    }

    fn apply_uniforms_from_bytes(&mut self, uniform_ptr: *const u8, size: usize) {
        assert!(
            self.current_pipeline.is_some(),
            "apply_uniforms before apply_pipeline"
        );
        assert!(
            self.render_encoder.is_some(),
            "apply_uniforms before begin_pass"
        );

        let current_pipeline = &self.pipelines[self.current_pipeline.unwrap().0];
        let render_encoder = self.render_encoder.unwrap();

        self.current_frame_index = (self.current_frame_index + 1) % NUM_INFLIGHT_FRAMES;

        let shader = &self.shaders[current_pipeline.shader.0];

        let data_lenght = shader.stride;

        assert!(size < MAX_UNIFORM_BUFFER_SIZE as usize);

        assert!(self.current_ub_offset < MAX_UNIFORM_BUFFER_SIZE);

        let buffer = self.uniform_buffers[self.current_frame_index];
        unsafe {
            let dest: *mut std::ffi::c_void = msg_send![buffer, contents];
            std::ptr::copy(
                uniform_ptr as _,
                dest.add(self.current_ub_offset as usize),
                size,
            );

            #[cfg(target_os = "macos")]
            msg_send_![buffer, didModifyRange:NSRange::new(0, size as u64)];

            msg_send_![render_encoder,
                       setVertexBuffer:buffer
                       offset:self.current_ub_offset
                       atIndex:0];
            msg_send_![render_encoder,
                       setFragmentBuffer:buffer
                       offset:self.current_ub_offset
                       atIndex:0];
        }
        self.current_ub_offset = roundup_ub_buffer(self.current_ub_offset + size as u64);
    }

    fn begin_default_pass(&mut self, action: PassAction) {
        self.begin_pass(None, action)
    }

    fn begin_pass(&mut self, pass: Option<RenderPass>, action: PassAction) {
        unsafe {
            if self.command_buffer.is_none() {
                self.command_buffer = Some(msg_send![self.command_queue, commandBuffer]);
            }

            let (descriptor, w, h) = match pass {
                None => {
                    let (screen_width, screen_height) = crate::window::screen_size();
                    (
                        {
                            let a = msg_send_![self.view, currentRenderPassDescriptor];
                            msg_send_![a, retain];
                            a
                        },
                        screen_width as f64,
                        screen_height as f64,
                    )
                }
                Some(pass) => {
                    //self.current_pass = Some(pass);

                    let pass_internal = &self.passes[pass.0];
                    (
                        pass_internal.render_pass_desc,
                        self.textures[pass_internal.texture.0].width as f64,
                        self.textures[pass_internal.texture.0].height as f64,
                    )
                }
            };
            assert!(!descriptor.is_null());

            let color_attachments = msg_send_![descriptor, colorAttachments];
            let color_attachment = msg_send_![color_attachments, objectAtIndexedSubscript: 0];

            msg_send_![color_attachment, setStoreAction: MTLStoreAction::Store];

            match action {
                PassAction::Clear {
                    color,
                    depth,
                    stencil,
                } => {
                    msg_send_![color_attachment, setLoadAction: MTLLoadAction::Clear];

                    if let Some(color) = color {
                        msg_send_![color_attachment, setClearColor:MTLClearColor::new(color.0 as _, color.1 as _, color.2 as _, color.3 as _)];
                    }
                }
                PassAction::Nothing => {
                    msg_send_![color_attachment, setLoadAction: MTLLoadAction::Load];
                }
            }

            let render_encoder = msg_send_![
                self.command_buffer.unwrap(),
                renderCommandEncoderWithDescriptor: descriptor
            ];

            // render_encoder.set_viewport(MTLViewport {
            //     originX: 0.0,
            //     originY: 0.0,
            //     width: w,
            //     height: h,
            //     znear: 0.0,
            //     zfar: 1.0,
            // });
            // render_encoder.set_scissor_rect(MTLScissorRect {
            //     x: 0,
            //     y: 0,
            //     width: w as u64,
            //     height: h as u64,
            // });

            self.render_encoder = Some(render_encoder);
        }
    }

    fn end_render_pass(&mut self) {
        assert!(
            self.render_encoder.is_some(),
            "end_render_pass unpaired with begin_render_pass!"
        );

        let render_encoder = self.render_encoder.unwrap();
        unsafe { msg_send_!(render_encoder, endEncoding) };

        self.render_encoder = None;
        self.index_buffer = None;
    }

    fn draw(&self, base_element: i32, num_elements: i32, num_instances: i32) {
        assert!(self.render_encoder.is_some(), "draw before begin_pass!");
        let render_encoder = self.render_encoder.unwrap();
        assert!(self.index_buffer.is_some());
        let index_buffer = self.index_buffer.unwrap();

        assert!(base_element == 0); // TODO: figure indexBufferOffset/baseVertex
        unsafe {
            msg_send_![render_encoder, drawIndexedPrimitives:MTLPrimitiveType::Triangle
                       indexCount:num_elements as u64
                       indexType:MTLIndexType::UInt16
                       indexBuffer:index_buffer
                       indexBufferOffset:0
                       instanceCount:num_instances as u64
                       baseVertex:0
                       baseInstance:0
            ];
        }
    }

    fn commit_frame(&mut self) {
        unsafe {
            assert!(!self.command_queue.is_null());
            let drawable: ObjcId = msg_send!(self.view, currentDrawable);
            msg_send_![drawable, retain];
            msg_send_![self.command_buffer.unwrap(), presentDrawable: drawable];
            msg_send_![self.command_buffer.unwrap(), commit];
            msg_send_![self.command_buffer.unwrap(), waitUntilCompleted];
        }
        for buffer in &mut self.buffers {
            buffer.next_value = 0;
        }
        self.current_ub_offset = 0;
        self.current_pipeline = None;
        self.command_buffer = None;
        if (self.current_frame_index + 1) >= 3 {
            self.current_frame_index = 0;
        }
    }
}
