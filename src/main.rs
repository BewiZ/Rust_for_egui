use eframe::egui::{self, FontData, FontDefinitions, TextureOptions, ScrollArea};
use std::path::Path;
use egui::{ColorImage, RichText, TextureHandle};
use nalgebra::Vector4;

struct MyEguiApp {
    selected_file: Option<String>,
    texture: Option<TextureHandle>,
    image_size: egui::Vec2,
    left_panel_width: f32,
    rows: Vec<TableRow>,
}

#[derive(Default)]
struct TextOptions {
    size: Option<f32>,
    color: Option<Vector4<u8>>,
    align: &'static str,
}

#[derive(Clone)]
struct TableRow {
    tag_name1: String,
    tag_value1: String,
    tag_name2: String,
    tag_value2: String,
}

impl Default for TableRow {
    fn default() -> Self {
        Self {
            tag_name1: String::new(),
            tag_value1: String::new(),
            tag_name2: String::new(),
            tag_value2: String::new(),
        }
    }
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            selected_file: None,
            texture: None,
            image_size: egui::Vec2::new(0.0, 0.0),
            left_panel_width: 0.0,
            rows: vec![
                TableRow {
                    tag_name1: "Make".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "Model".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "Orientation".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "X Resolution".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "Y Resolution".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "Resolution Unit".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "Software".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "Modify Date".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "Y Cb Cr Positioning".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "Exposure Time".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "F Number".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "Exposure Program".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "ISO".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "Sensitivity Type".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
                TableRow {
                    tag_name1: "".to_string(),
                    tag_value1: "".to_string(),
                    tag_name2: "".to_string(),
                    tag_value2: "".to_string(),
                },
            ],
        }
    }
}
/**
 * 设置 EGUI 字体的函数
 * @param ctx - EGUI 上下文引用，用于设置字体
 */
fn setup_fonts_and_style(ctx: &egui::Context) {
    // 创建一个新的字体定义，使用默认配置
    let mut fonts = FontDefinitions::default();

    // 从系统字体加载中文字体（这里以 "微软雅黑" 为例），将字体文件数据插入到字体定义中
    fonts.font_data.insert(
        "微软雅黑".to_owned(), // 字体名称
        std::sync::Arc::new(
            FontData::from_static(
                include_bytes!(
                    "C:\\Windows\\Fonts\\msyh.ttc" // 替换为你的字体路径
                )
            )
        ), 
    );

    // 将中文字体添加到默认字体族中
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
        .insert(0, "微软雅黑".to_owned());
   
    ctx.set_fonts(fonts); // 应用新的字体

    let mut style = (*ctx.style()).clone(); // 克隆当前样式

    // 设置默认文本样式
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Button, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Small, egui::FontId::new(10.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Monospace)),
    ].into();

    // 设置默认文本颜色
    style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255);
    style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255);
    style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(200, 200, 200);
    style.visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(150, 150, 150);
    
    // 设置背景颜色
    style.visuals.window_fill = egui::Color32::from_rgb(45, 45, 45);
    style.visuals.panel_fill = egui::Color32::from_rgb(30, 30, 30);
    style.visuals.window_stroke.color = egui::Color32::from_rgb(80, 80, 80);

    // 设置超链接颜色
    style.visuals.hyperlink_color = egui::Color32::from_rgb(100, 150, 255);
    
    ctx.set_style(style); // 应用新的样式
}

fn custom_text(
    ui: &mut egui::Ui,
    text: &str,
    heading_or_label: &str,
    options: Option<TextOptions>,
) {
    // 设置默认值
    let is_heading = match heading_or_label {
        "heading" => true,
        "label" => false,
        _ => false,
    };

    let options = options.unwrap_or_default();
    let size = match options.size {
        Some(size) => size,
        None => 16.0,
    };


    let color = match options.color {
        Some(color) => color,
        None => Vector4::new(255, 255, 255, 255),
    };

    let align = match options.align {
        "LEFT" => egui::Align::LEFT,
        "CENTER" => egui::Align::Center,
        "RIGHT" => egui::Align::RIGHT,
        _ => egui::Align::LEFT,
    };

    // 创建富文本
    let rich_text = egui::RichText::new(text)
        .size(size)
        .color(egui::Color32::from_rgba_premultiplied(
            color.x,
            color.y,
            color.z,
            color.w,
        ));

    // 根据对齐方式设置布局
    ui.with_layout(egui::Layout::top_down(align),|ui| {
        if is_heading {
            ui.heading(rich_text);
        } else {
            ui.label(rich_text);
        }
    }).inner;

}

impl MyEguiApp {
    fn load_image(&mut self, ctx: &egui::Context, path: &str) -> Result<(), String> {
        // 读取图片文件到字节数组
        // 使用std::fs::read读取整个文件到内存
        let image_bytes = std::fs::read(path)
            .map_err(|e| format!("无法读取文件: {}", e))?;
        
        // 解码图片字节数据
        // 使用image库从内存字节加载图片，支持多种格式(PNG, JPG, JPEG, BMP, GIF等)
        let image = image::load_from_memory(&image_bytes)
            .map_err(|e| format!("无法解码图片: {}", e))?
            .to_rgba8();
        
        // 获取图片尺寸
        // image.width()和image.height()返回u32类型，转换为usize用于数组索引
        let size = [image.width() as _, image.height() as _];

        // 创建EGUI颜色图像数据
        // from_rgba_unmultiplied: 从非预乘RGBA数据创建颜色图像
        // 预乘alpha意味着颜色值已经乘以了alpha值，这里使用非预乘格式
        let image_data = ColorImage::from_rgba_unmultiplied(size, &image);
        
        // 存储图片尺寸到应用状态中，用于后续显示比例计算
        // 转换为f32类型，因为EGUI使用浮点数坐标系统
        self.image_size = egui::Vec2::new(size[0] as f32, size[1] as f32);
        
        // 从文件路径提取文件名作为纹理名称
        // 如果无法提取文件名，使用默认名称"image"
        let texture_name = Path::new(path)
            .file_name() // 获取文件名部分(不含路径)
            .and_then(|n| n.to_str()) // OsStr转换为&str
            .unwrap_or("image"); // 如果转换失败使用默认值

        self.texture = Some(ctx.load_texture(
            texture_name, // 纹理标识名称
            image_data, // 图片数据
            TextureOptions::default() // 纹理选项
        ));

        Ok(())
    }



}

impl eframe::App for MyEguiApp {
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let available_width = ctx.available_rect().width(); // 使用 ctx 获取宽度
        let available_height = ctx.available_rect().height(); // 使用 ctx 获取高度

        // let left_panel_width = available_width * 0.35;  // 与左侧面板的default_width保持一致
        // 左侧可滚动面板
        egui::SidePanel::left("left_panel")
            .resizable(true)  // 允许调整大小
            .default_width(available_width * 0.35)  // 默认宽度
            .min_width(available_width * 0.3)  // 最小宽度
            .max_width(available_width * 0.4)  // 最大宽度
            .show(ctx, |ui| {

                self.left_panel_width = ui.available_width();  // 更新实际宽度

                custom_text(ui, "Area can scroll on the left", "heading", 
                Some(TextOptions { 
                    size: Some(20.0), 
                    color: Some(Vector4::new(180, 200, 150, 255)),
                    align: "CENTER"
                }));

                ui.hyperlink_to("EXIF Viewer", "https://exifviewer.com/zh/");

                ui.separator();
                
                // 添加滚动区域
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if ui.button("Files").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Picture", &["png", "jpg", "jpeg", "bmp", "gif"])
                                .pick_file() 
                            {
                                let file_path = path.display().to_string();
                                self.selected_file = Some(file_path.clone());
                                        
                                // 加载选中的图片
                                if let Err(e) = self.load_image(ctx, &file_path) {
                                    eprintln!("Fail to load pictures: {}", e);
                                            self.texture = None;
                                }
                            }
                        }

                        ui.separator(); // 添加分隔线
                        if self.rows.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("No data available, please add new items above");
                            });
                            return;
                        }

                        egui::Grid::new("products_grid")
                            .num_columns(4)
                            .spacing([20.0, 15.0]) // 行、列间距
                            .striped(true) // 斑马纹
                            .min_col_width(40.0)
                            .show(ui, |ui| {
                                // 表头
                                ui.heading(RichText::new("标签名称").size(14.0));
                                ui.heading(RichText::new("值").size(14.0));
                                ui.heading(RichText::new("标签名称").size(14.0));
                                ui.heading(RichText::new("值").size(14.0));
                                ui.end_row();

                                for (i, row) in self.rows.iter_mut().enumerate() { // 每行遍历添加
                                    // 第一列: 固定文本
                                    ui.label(RichText::new(&row.tag_name1).size(12.0));
                                    // 第二列: 可编辑
                                    ui.text_edit_singleline(&mut row.tag_value1);
                                    // 第三列: 固定文本
                                    ui.label(RichText::new(&row.tag_name2).size(12.0));
                                    // 第四列: 可编辑
                                    ui.text_edit_singleline(&mut row.tag_value2);
                                    ui.end_row();
                                }
                            }
                        );

                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(format!("总行数: {}", self.rows.len()));
                            if ui.button("重置").clicked() {
                                *self = MyEguiApp::default();
                            }
                        });
                    }
                );
            }
        );
        // 右侧固定面板
        egui::CentralPanel::default()
            .show(ctx, |ui| {

                // 右侧内容 - 不会滚动
                ui.vertical_centered(|ui| {
                    ui.set_min_height(ui.available_height());
                    
                    // 显示图片
                    if let Some(texture) = &self.texture {

                        custom_text(ui, "Preview:", "heading",
                        Some(TextOptions {
                            size: Some(24.0),
                            color: None,
                            align: "LEFT",
                        }));
                        ui.separator();

                        let rect_w = ui.available_width();
                        let start_pos = 1200.0 - rect_w;
                        
                        // 计算适合的显示尺寸
                        // pic_width 和 pic_height 分别是当前可用宽度和高度的90%
                        // display_width 和 display_height 是根据图片宽高比计算出的显示尺寸
                        let pic_zoom = 0.9;
                        let (display_width, display_height);
                        if self.image_size.x > self.image_size.y {
                            let pic_width = ui.available_width();
                            display_width = pic_width * pic_zoom;
                            let img_ratio = self.image_size.x / self.image_size.y;
                            display_height = display_width / img_ratio;
                        } else {
                            let pic_height = ui.available_height();
                            display_height = pic_height * pic_zoom;
                            let img_ratio = self.image_size.y / self.image_size.x;
                            display_width = display_height / img_ratio;
                        }
                        let display_size = egui::Vec2::new(display_width, display_height);
                        
                        // 创建固定高度的容器并显示图片
                        ui.scope_builder(
                            egui::UiBuilder::new()
                                .max_rect(egui::Rect::from_min_size(
                                    ui.available_rect_before_wrap().min,
                                    egui::vec2(ui.available_width() * pic_zoom, ui.available_height() * pic_zoom)
                                )),
                            |ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.add(egui::Image::from_texture(texture).fit_to_exact_size(display_size));
                                });
                            }
                        );

                        ui.separator();

                        let rect_h = ui.available_height();
                        let end_pos = 800.0 - rect_h;

                        // 显示图片信息
                        ui.scope_builder(
                            egui::UiBuilder::new()
                                .max_rect(egui::Rect::from_min_size(
                                    egui::pos2(start_pos, end_pos),
                                    egui::vec2(rect_w, ui.available_height())
                                )),
                            |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.label(format!("Image size: {} x {}", self.image_size.x, self.image_size.y));
                                    if let Some(file) = &self.selected_file {
                                        ui.label(egui::RichText::new(format!("File: {}", file))
                                            .small()
                                            .color(egui::Color32::LIGHT_GRAY));
                                    }
                                });
                            }
                        );

                    } else {
                        // 没有图片时的占位内容 - 使用垂直居中
                        ui.vertical_centered(|ui| {
                            ui.add_space(available_height * 0.3);
                            ui.heading(egui::RichText::new("Please select a picture").size(48.0));
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new("Support: PNG, JPG, JPEG, BMP, GIF").size(20.0));
                        });
                    }
                });
            });
    }
}



fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]) // 窗口大小
            .with_resizable(false), // 禁止窗口缩放
        ..Default::default()
    };


    let _ = eframe::run_native(
        "My egui App", 
        native_options, 
        Box::new(|cc| {
            setup_fonts_and_style(&cc.egui_ctx); // 设置自定义字体和样式
            Ok(Box::new(MyEguiApp::default())) // 创建并返回 MyEguiApp 实例
        })
    );
}