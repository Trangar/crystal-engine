#[cfg(feature = "format-fbx")]
pub use crate::model::FbxError;
#[cfg(feature = "format-obj")]
pub use crate::model::ObjError;

use thiserror::Error;

/// Errors generated when loading a model
#[derive(Error, Debug)]
pub enum ModelError {
    /// Could not load a texture from the given path
    #[error("Could not load texture {path:?}: {inner:?}")]
    CouldNotLoadTexture {
        /// The path of the texture that was trying to be loaded
        path: String,
        /// The inner exception that occured when loading the texture
        inner: image::error::ImageError,
    },

    /// The model that was loaded has an invalid model vertex buffer
    #[error("Model has no valid vertex buffer")]
    InvalidModelVertexBuffer,

    /// The error that was thrown whilst loading an .obj file.
    ///
    /// This error can only be thrown if the `format-obj` feature is enabled
    #[cfg(feature = "format-obj")]
    #[error("Could not load OBJ model: {0:?}")]
    Obj(ObjError),

    /// The error that was thrown whilst loading an .fbx file.
    ///
    /// This error can only be thrown if the `format-fbx` feature is enabled
    #[cfg(feature = "format-fbx")]
    #[error("Could not load FBX model: {0:?}")]
    Fbx(FbxError),
}

/// Errors generated when creating GUI elements
#[derive(Error, Debug)]
pub enum GuiError {
    /// Could not load the texture at the given path
    #[error("Could not load texture {path:?}: {inner:?}")]
    CouldNotLoadTexture {
        /// The path where the texture was loaded from
        path: String,
        /// The inner error that was thrown
        inner: image::error::ImageError,
    },
    /// Could not turn the texture into a vulkano image
    #[error("Could not create texture: {inner:?}")]
    CouldNotCreateTexture {
        /// The inner error
        inner: vulkano::image::ImageCreationError,
    },
    /// Could not read the given font
    #[error("Could not read font file {file:?}: {inner:?}")]
    CouldNotReadFontFile {
        /// The inner error
        inner: std::io::Error,
        /// The file being loaded
        file: String,
    },
    /// Could not parse the font file
    #[error("Could not load font")]
    CouldNotLoadFont,
}

/// Errors that are thrown during initialization. These are mostly internal and graphic card errors and are (hopefully) unlikely to occur.
#[derive(Error, Debug)]
pub enum InitError {
    /// Could not load the capabilities of a surface
    #[error("Could not load surface capabilities: {0:?}")]
    CouldNotLoadSurfaceCapabilities(vulkano::swapchain::CapabilitiesError),

    /// Could not load the alpha channel of the surface
    #[error("The selected surface has no support for alpha blending")]
    NoCompositeAlpha,

    /// Could not initialize the swapchain
    #[error("Could not initialize the swapchain: {0:?}")]
    CouldNotInitSwapchain(vulkano::swapchain::SwapchainCreationError),

    /// Could not create the swapchain images
    #[error("Could not create swapchain images: {0:?}")]
    CouldNotBuildSwapchainImages(vulkano::framebuffer::FramebufferCreationError),

    /// Could not recreate the swapchain images, which usually happens on resizing the window
    #[error("Could not recreate the swapchain: {0:?}")]
    CouldNotRecreateSwapchain(vulkano::swapchain::SwapchainCreationError),

    /// Could not acquire the next swapchain image
    #[error("Could not acquire the next swapchain image: {0:?}")]
    CouldNotAcquireSwapchainImage(vulkano::swapchain::AcquireError),

    /// Could not create a vulkano device
    #[error("Could not create a device")]
    CouldNotCreateDevice(vulkano::device::DeviceCreationError),

    /// Could not find a physical device
    #[error("Could not find a physical device")]
    CouldNotFindPhysicalDevice,

    /// Could not find a valid graphics queue
    #[error("Could not find a valid graphics queue")]
    CouldNotFindValidGraphicsQueue,

    /// Could not initialize Vulkano
    #[error("Could not init Vulkano: {0:?}")]
    CouldNotInitVulkano(vulkano::instance::InstanceCreationError),

    /// Could not create a vulkano_win window
    #[error("Could not create a window: {0:?}")]
    CouldNotCreateWindow(vulkano_win::CreationError),
}
