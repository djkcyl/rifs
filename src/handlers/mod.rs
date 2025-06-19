pub mod image_handler;
pub mod static_files;
 
pub use image_handler::{
    health_check, upload_image, get_image, get_image_info, 
    query_images_post, query_images_get, get_stats, delete_image
};
pub use static_files::api_docs; 