pub mod image_handler;
pub mod cache_handler;
pub mod static_files;
pub mod health_handler;

pub use image_handler::{
    upload_image, get_image, get_image_info,
    query_images_post, query_images_get, get_stats, delete_image,
};
pub use cache_handler::{
    get_cache_stats, auto_cleanup_cache, cleanup_cache_with_policy,
    clear_all_cache, cache_management_dashboard, smart_cleanup, decay_heat_scores,
    smart_space_cleanup,
};
pub use static_files::api_docs;
pub use health_handler::{health_check_detailed, get_system_stats}; 