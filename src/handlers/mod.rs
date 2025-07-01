pub mod cache_handler;
pub mod health_handler;
pub mod image_handler;
pub mod static_files;

pub use cache_handler::{
    auto_cleanup_cache, cache_management_dashboard, clear_all_cache, decay_heat_scores,
    get_cache_stats,
};
pub use health_handler::{get_system_stats, health_check_detailed};
pub use image_handler::{
    delete_image, get_image, get_image_info, get_stats, query_images_get, query_images_post,
    upload_image,
};
pub use static_files::api_docs;
