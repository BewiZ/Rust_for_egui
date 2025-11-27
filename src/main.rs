extern crate exif;

use eframe::egui::{
    self, 
    FontData, 
    FontDefinitions, 
    TextureOptions, 
    ScrollArea,
    ComboBox
};
use std::{
    path::Path,
    fs::File,
    io::{BufReader, BufRead, Seek, SeekFrom},
    collections::HashMap,
    env,
    time::Instant,
};
use egui::{ColorImage, RichText, TextureHandle};
use nalgebra::Vector4;

use exif::{Exif, In, Reader, Tag, Value};
use rusttype::{Font, Scale, point};

struct MyEguiApp {
    selected_file: Option<String>,
    texture: Option<TextureHandle>,
    image_size: egui::Vec2,
    left_panel_width: f32,
    rows: Vec<TableRow>,
    default_rows: Vec<TableRow>,
    initial_exif_data: Vec<TableRow>, // 新增：保存第一次加载图片时的EXIF数据

    up_value: u32,
    down_value: u32,
    left_value: u32,
    right_value: u32,
    min_value: f64,
    max_value: f64,
    step: f64,
    decimal_places: usize,
    bg_color: egui::Color32, // 新增：背景颜色
    enable_blur_bg: bool, // 新增：是否启用模糊背景
    blur_strength: f32, // 新增：模糊强度

    show_custom_bg_color_picker: bool, // 新增：是否显示自定义背景颜色选择器

    original_image: Option<image::DynamicImage>,
    export_toast: Option<String>,       // 提示文本
    export_toast_is_success: bool,      // 是否成功提示
    toast_timer: Option<std::time::Instant>, // 新增：用于跟踪吐司显示时间
}

#[derive(Default)]
struct TextOptions {
    size: Option<f32>,
    color: Option<Vector4<u8>>,
    align: &'static str,
}

#[derive(Clone)]
struct TableRow {
    tag_name: String,
    tag_value: String,
}

impl Default for TableRow {
    fn default() -> Self {
        Self {
            tag_name: String::new(),
            tag_value: String::new(),
        }
    }
}

impl Default for MyEguiApp {
    fn default() -> Self {

        let default_rows = vec![
            TableRow {
                tag_name: "相机型号".to_string(),
                tag_value: "".to_string(),
            },
            TableRow {
                tag_name: "图像宽度".to_string(),
                tag_value: "".to_string(),
            },
            TableRow {
                tag_name: "图像高度".to_string(),
                tag_value: "".to_string(),
            },
            TableRow {
                tag_name: "ISO".to_string(),
                tag_value: "".to_string(),
            },
            TableRow {
                tag_name: "光圈".to_string(),
                tag_value: "".to_string(),
            },
            TableRow {
                tag_name: "曝光时长".to_string(),
                tag_value: "".to_string(),
            },
            TableRow {
                tag_name: "焦距".to_string(),
                tag_value: "".to_string(),
            },
            TableRow {
                tag_name: "日期".to_string(),
                tag_value: "".to_string(),
            },
            TableRow {
                tag_name: "时间".to_string(),
                tag_value: "".to_string(),
            },
        ];
        Self {
            selected_file: None,
            texture: None,
            image_size: egui::Vec2::new(0.0, 0.0),
            left_panel_width: 0.0,
            rows: default_rows.clone(), // 使用默认行初始化
            default_rows, // 保存备份
            initial_exif_data: Vec::new(), // 初始为空向量

            up_value: 0,
            down_value: 75,
            left_value: 0,
            right_value: 0,
            min_value: 0.0,
            max_value: 100.0,
            step: 1.0,
            decimal_places: 1,
            bg_color: egui::Color32::from_rgba_premultiplied(255, 255, 255, 255), // 默认白色背景
            enable_blur_bg: false, // 默认不启用模糊背景
            blur_strength: 1.0, // 默认模糊强度
            show_custom_bg_color_picker: false, // 默认不显示自定义背景颜色选择器
            original_image: None,
            export_toast: None,
            export_toast_is_success: false,
            toast_timer: None,
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
    fonts.font_data.insert(
        "Raleway_ExtraBold_Italic".to_owned(), // 字体名称
        std::sync::Arc::new(
            FontData::from_static(
                include_bytes!(
                    "C:\\Windows\\Fonts\\Raleway-Italic-VariableFont_wght.ttf"
                )
            )
        ), 
    );

    // 将中文字体添加到默认字体族中
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
        .insert(0, "微软雅黑".to_owned());
   
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
        .insert(1, "Raleway_ExtraBold_Italic".to_owned());

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


fn analyse_jpg(exif: &Exif, row: &mut TableRow) {
   match row.tag_name.as_str() {
       "相机型号" => {
           if let Some(field) = exif.get_field(Tag::Model, In::PRIMARY) {
               if let Value::Ascii(ref vec) = field.value {
                   if !vec.is_empty() {
                       row.tag_value = String::from_utf8_lossy(&vec[0]).into_owned();
                   } else {
                       row.tag_value = field.display_value().to_string();
                   }
               } else {
                   row.tag_value = field.display_value().to_string();
               }
           }
       },
       "图像宽度" => {
           if let Some(field) = exif.get_field(Tag::PixelXDimension, In::PRIMARY) {
               row.tag_value = field.display_value().to_string();
           }
       },
       "图像高度" => {
           if let Some(field) = exif.get_field(Tag::PixelYDimension, In::PRIMARY) {
               row.tag_value = field.display_value().to_string();
           }
       },
       "ISO" => {
           if let Some(field) = exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY) {
               row.tag_value = field.display_value().with_unit(exif).to_string();
           }
       },
       "光圈" => {
           if let Some(field) = exif.get_field(Tag::FNumber, In::PRIMARY) {
               row.tag_value = field.display_value().with_unit(exif).to_string();
           }
       },
       "曝光时长" => {
           if let Some(field) = exif.get_field(Tag::ExposureTime, In::PRIMARY) {
               row.tag_value = field.display_value().with_unit(exif).to_string();
           }
       },
       "焦距" => {
           if let Some(field) = exif.get_field(Tag::FocalLength, In::PRIMARY) {
               row.tag_value = field.display_value().with_unit(exif).to_string();
           }
       }
       "日期" | "时间" => {
           if let Some(field) = exif.get_field(Tag::DateTime, In::PRIMARY) {
               let datetime_str = field.display_value().to_string();
               let mut datetime_parts = datetime_str.split_whitespace();
               if let (Some(date), Some(time)) = (datetime_parts.next(), datetime_parts.next()) {
                   if row.tag_name == "日期" {
                       let formatted_date = date.replace(":", "-");
                       row.tag_value = formatted_date;
                   } else if row.tag_name == "时间" {
                       row.tag_value = time.to_string();
                   }
               }
           }
       },
       _ => {}
   }
}

// 格式化光圈值
fn format_f_number(value: &str) -> String {
    if let Some(devide_i) = value.find('/') {
        let devidend = &value[..devide_i]; // 被除数
        let devisor = &value[devide_i+1..]; // 除数
        let quotient = devidend.parse::<f32>().unwrap_or(0.0) / devisor.parse::<f32>().unwrap_or(1.0);
        format!("f/{:.1}", quotient)
    } else {
        value.to_string()
    }
}

// 格式化焦距
fn format_focal_length(value: &str) -> String {
    if let Some(devide_i) = value.find('/') {
        let devidend = &value[..devide_i]; // 被除数
        let devisor = &value[devide_i+1..]; // 除数
        let quotient = devidend.parse::<f32>().unwrap_or(0.0) / devisor.parse::<f32>().unwrap_or(1.0);
        format!("{}mm", quotient)
    } else {
        value.to_string()
    }
}

// 格式化曝光时长
fn format_exposure_time(value: &str) -> String {
    if let Some(devide_i) = value.find('/') {
        let devidend = &value[..devide_i]; // 被除数
        let devisor = &value[devide_i+1..]; // 除数
        let quotient = devidend.parse::<f32>().unwrap_or(0.0) / devisor.parse::<f32>().unwrap_or(1.0);

        if quotient < 1.0 {
            format!("1/{}", (1.0/quotient).round() as u32)
        } else {println!("{}", quotient);
            format!("{}s", quotient)
        }
    } else {
        value.to_string()
    }
}

fn format_datetime(value: &str) -> String {
    // 处理 ISO 8601 格式，如 "2025-11-03T21:23:56.26+08:00"
    if let Some(t_index) = value.find('T') {
        let date_part = &value[..t_index];
        let time_part = &value[t_index+1..];
        if let Some(plus_index) = time_part.find('+') {
            let time_without_tz = &time_part[..plus_index];
            return format!("{} {}", date_part, time_without_tz);
        }
    }
    value.to_string()
}

// 添加专门处理 RDF/XML 序列格式的通用函数
fn extract_value_from_rdf_sequence(xmp_data: &str, tag_name: &str) -> Option<String> {
    // 查找序列的开始标签
    let start_patterns = [
        format!("<{}>", tag_name),
        format!("<{} ", tag_name), // 处理带属性的标签
    ];
    
    for pattern in &start_patterns {
        if let Some(start) = xmp_data.find(pattern) {
            // 找到序列开始位置
            let seq_start = start + pattern.len();
            
            // 查找序列结束标签
            let end_pattern = format!("</{}>", tag_name);
            if let Some(end) = xmp_data[seq_start..].find(&end_pattern) {
                let sequence_content = &xmp_data[seq_start..seq_start + end];
                
                // 在序列内容中查找 <rdf:li> 标签
                if let Some(li_start) = sequence_content.find("<rdf:li>") {
                    let value_start = li_start + "<rdf:li>".len();
                    if let Some(li_end) = sequence_content[value_start..].find("</rdf:li>") {
                        let value = &sequence_content[value_start..value_start + li_end];
                        if !value.trim().is_empty() {
                            return Some(value.trim().to_string());
                        }
                    }
                }
                
                // 如果没有找到 <rdf:li> 标签，尝试直接获取标签内的文本内容
                if let Some(text_end) = sequence_content.find('<') {
                    let value = &sequence_content[..text_end];
                    if !value.trim().is_empty() {
                        return Some(value.trim().to_string());
                    }
                }
            }
        }
    }
    
    None
}

// 从 XMP 数据中提取 EXIF 信息
fn extract_exif_from_xmp(xmp_data: &str) -> Option<HashMap<String, String>> {
    // 这些是常见的 XMP/EXIF 标签及其对应的表格字段名
    let tags = [
        ("tiff:Model", "相机型号"),
        ("exif:Model", "相机型号"),
        ("tiff:Make", "相机制造商"),
        ("exif:Make", "相机制造商"),
        ("exif:DateTimeOriginal", "拍摄时间"),
        ("exif:ExposureTime", "曝光时长"),
        ("exif:FNumber", "光圈"),
        ("exif:FocalLength", "焦距"),
        ("exif:ISOSpeedRatings", "ISO"),
        ("tiff:ImageWidth", "图像宽度"),
        ("tiff:ImageLength", "图像高度"),
        ("exif:PixelXDimension", "图像宽度"),
        ("exif:PixelYDimension", "图像高度"),
    ];
    
    let mut exif_data = HashMap::new();
    let mut found_data = false;
    
    for (tag, field_name) in &tags {
        let mut value_found = false;
        
        // 首先尝试从属性中提取值
        if let Some(start) = xmp_data.find(&format!("{}=\"", tag)) {
            let value_start = start + tag.len() + 2; // 跳过 tag="
            if let Some(end) = xmp_data[value_start..].find('"') {
                let value = &xmp_data[value_start..value_start + end];
                if !value.is_empty() {
                    // 格式化输出
                    let formatted_value = match *field_name {
                        "光圈" => format_f_number(value),
                        "焦距" => format_focal_length(value),
                        "曝光时长" => format_exposure_time(value),
                        "拍摄时间" => format_datetime(value),
                        _ => value.to_string(),
                    };
                    println!("{}: {}", field_name, formatted_value);
                    exif_data.insert(field_name.to_string(), formatted_value);
                    found_data = true;
                    value_found = true;
                }
            }
        }
        
        // 如果没有从属性中找到值，尝试从 RDF/XML 序列中提取
        if !value_found {
            if let Some(value) = extract_value_from_rdf_sequence(xmp_data, tag) {
                // 格式化输出
                let formatted_value = match *field_name {
                    "光圈" => format_f_number(&value),
                    "焦距" => format_focal_length(&value),
                    "曝光时长" => format_exposure_time(&value),
                    "拍摄时间" => format_datetime(&value),
                    _ => value.to_string(),
                };
                println!("Found {} in RDF sequence: {}", field_name, formatted_value);
                exif_data.insert(field_name.to_string(), formatted_value);
                found_data = true;
            }
        }
    }
    
    if found_data {
        Some(exif_data)
    } else {
        None
    }
}

// PNG 相关常量
const PNG_SIG: [u8; 8] = *b"\x89PNG\x0d\x0a\x1a\x0a";
const EXIF_CHUNK_TYPE: [u8; 4] = *b"eXIf";
const ITXT_CHUNK_TYPE: [u8; 4] = *b"iTXt";
const IHDR_CHUNK_TYPE: [u8; 4] = *b"IHDR";


fn get_png_exif<R>(reader: &mut R) -> Result<(Vec<u8>, Option<(u32, u32)>, Option<HashMap<String, String>>), Box<dyn std::error::Error>>
where
    R: BufRead + Seek,
{
    let mut sig = [0u8; 8];
    reader.read_exact(&mut sig)?;
    if sig != PNG_SIG {
        return Err("Not a PNG file".into());
    }

    let mut exif_data = Vec::new();
    let mut dimensions = None;
    let mut xmp_exif_data = None;
    
    loop {
        // 读取块长度
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).is_err() {
            break;
        }
        let length = u32::from_be_bytes(len_buf) as usize;

        // 读取块类型
        let mut chunk_type = [0u8; 4];
        if reader.read_exact(&mut chunk_type).is_err() {
            break;
        }

        // 检查是否是 IHDR 块（包含图像宽高）
        if chunk_type == IHDR_CHUNK_TYPE {
            let mut data = vec![0u8; length];
            reader.read_exact(&mut data)?;
            
            // 解析宽高信息
            if data.len() >= 8 {
                let width = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                let height = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
                dimensions = Some((width, height));
            }
            
            // 跳过 CRC
            let mut crc = [0u8; 4];
            let _ = reader.read_exact(&mut crc);
        }
        // 检查是否是 EXIF 块
        else if chunk_type == EXIF_CHUNK_TYPE {
            let mut data = vec![0u8; length];
            reader.read_exact(&mut data)?;

            // 跳过 CRC
            let mut crc = [0u8; 4];
            let _ = reader.read_exact(&mut crc);

            exif_data = data;
            break; // 找到 EXIF 数据后可以提前退出
        } 
        // 检查是否是 iTXt 块
        else if chunk_type == ITXT_CHUNK_TYPE && xmp_exif_data.is_none() {
            let mut data = vec![0u8; length];
            reader.read_exact(&mut data)?;
            
            // 将 iTXt 数据转换为字符串进行分析
            if let Ok(text) = String::from_utf8(data) {
                // 检查是否包含 XMP 数据（通常包含 EXIF）
                if text.contains("xmp") || text.contains("XMP") || text.contains("exif") || text.contains("EXIF") {
                    // 尝试查找 XML 格式的 XMP 数据
                    if let Some(xmp_start) = text.find("<x:xmpmeta") {
                        if let Some(xmp_end) = text.find("</x:xmpmeta>") {
                            let xmp_data = &text[xmp_start..xmp_end + 12]; // +12 包含结束标签
                            
                            println!("Found XMP data in iTXt chunk:");
                            // 从 XMP 中提取基本的 EXIF 信息
                            xmp_exif_data = extract_exif_from_xmp(xmp_data);
                            // println!("{:?}", &xmp_data);
                        }
                    }
                }
            }
            
            // 跳过 CRC
            let mut crc = [0u8; 4];
            let _ = reader.read_exact(&mut crc);
        }
        else {
            // 跳过其他块的数据和 CRC
            let skip_length = length + 4;
            if reader.seek(SeekFrom::Current(skip_length as i64)).is_err() {
                break;
            }

            // 如果是 IEND 块，停止搜索
            if chunk_type == *b"IEND" {
                break;
            }
        }
    }

    // 如果找到了标准 EXIF 数据，返回它和图像尺寸
    if !exif_data.is_empty() {
        return Ok((exif_data, dimensions, None));
    }
    
    // 返回 XMP EXIF 数据和图像尺寸
    if dimensions.is_some() || xmp_exif_data.is_some() {
        Ok((Vec::new(), dimensions, xmp_exif_data))
    } else {
        Err("EXIF chunk not found".into())
    }
}

// 添加 analyse_png 函数定义
fn analyse_png(row: &mut TableRow, exif_data: &Option<Vec<u8>>, dimensions: &Option<(u32, u32)>, xmp_exif_data: &Option<HashMap<String, String>>) {
    // 首先尝试使用标准 EXIF 数据
    if let Some(data) = exif_data {
        match Reader::new().read_raw(data.clone()) {
            Ok(exif) => {
                // 使用与 JPEG 相同的分析逻辑
                analyse_jpg(&exif, row);
                // 即使有标准 EXIF 数据，也使用 IHDR 的尺寸信息（更准确）
                if let Some((width, height)) = dimensions {
                    if row.tag_name == "图像宽度" {
                        row.tag_value = format!("{} px", width);
                    } else if row.tag_name == "图像高度" {
                        row.tag_value = format!("{} px", height);
                    }
                }
                return;
            }
            Err(e) => {
                eprintln!("Failed to parse PNG EXIF data: {}", e);
            }
        }
    }
    
    // 如果标准 EXIF 数据不可用，尝试使用 XMP 数据
    if let Some(xmp_data) = xmp_exif_data {
        update_row_from_xmp(row, xmp_data);
        // 使用 XMP 数据后，也使用 IHDR 的尺寸信息
        if let Some((width, height)) = dimensions {
            if row.tag_name == "图像宽度" {
                row.tag_value = format!("{} px", width);
            } else if row.tag_name == "图像高度" {
                row.tag_value = format!("{} px", height);
            }
        }
        return;
    }
    
    // 最后，使用图像尺寸信息
    // if let Some((width, height)) = dimensions {
    //     if row.tag_name == "图像宽度" {
    //         row.tag_value = format!("{} px", width);
    //     } else if row.tag_name == "图像高度" {
    //         row.tag_value = format!("{} px", height);
    //     }
    // }
    
    // 如果以上都没有数据，设置为无数据
    if row.tag_value.is_empty() {
        row.tag_value = "无EXIF数据".to_string();
    }
}

fn update_row_from_xmp(row: &mut TableRow, xmp_data: &HashMap<String, String>) {
    // 直接根据行名称查找对应的值
    if let Some(value) = xmp_data.get(&row.tag_name) {
        row.tag_value = value.clone();
        return;
    }
    
    // 如果直接匹配失败，尝试其他可能的字段名
    match row.tag_name.as_str() {
        "相机型号" => {
            if let Some(value) = xmp_data.get("相机制造商") {
                row.tag_value = value.clone();
            }
        }
        "日期" => {
            if let Some(value) = xmp_data.get("拍摄时间") {
                // 从日期时间字符串中提取日期部分
                if let Some(date_part) = value.split_whitespace().next() {
                    row.tag_value = date_part.to_string();
                }
            }
        }
        "时间" => {
            if let Some(value) = xmp_data.get("拍摄时间") {
                // 从日期时间字符串中提取时间部分
                if let Some(time_part) = value.split_whitespace().nth(1) {
                    row.tag_value = time_part.to_string();
                }
            }
        }
        "ISO" => {
            // 尝试多种可能的 ISO 字段名
            let possible_iso_keys = ["ISO", "ISOSpeedRatings", "PhotographicSensitivity"];
            for key in &possible_iso_keys {
                if let Some(value) = xmp_data.get(*key) {
                    row.tag_value = value.clone();
                    println!("Set ISO to: {}", value);
                    return;
                }
            }
            println!("No ISO data found in XMP");
        }
        _ => {}
    }
}

impl MyEguiApp {
    fn load_image(&mut self, ctx: &egui::Context, path: &str) -> Result<(), String> {

        let start_time = std::time::Instant::now();  // 开始计时

        // 读取图片文件到字节数组
        // 使用std::fs::read读取整个文件到内存
        let image_bytes = std::fs::read(path)
            .map_err(|e| format!("无法读取文件: {}", e))?;
        
        // ========== 新增：保存原始图片数据（保留位深/像素） ==========
        let original_image = image::load_from_memory(&image_bytes)
            .map_err(|e| format!("无法解码图片: {}", e))?;
        self.original_image = Some(original_image.clone());
        
        // 转换为EGUI显示用的Rgba8格式
        let image = original_image.to_rgba8();
        // ========================================================

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

        self.read_exif(path);

        // 如果是第一次加载图片，保存初始EXIF数据
        if self.initial_exif_data.is_empty() {
            self.initial_exif_data = self.rows.clone();
        }

        let duration = start_time.elapsed();  // 计算耗时
        println!("图片加载耗时: {:?}", duration);  // 打印加载时间

        Ok(())
    }

    // ========== 新增：文字绘制辅助函数 ==========
    /// 将文字绘制到image::DynamicImage缓冲区
    fn draw_text_to_image(
        &self,
        image: &mut image::DynamicImage,
        font: &Font,
        scale: Scale,
        text: &str,
        pos: (f32, f32),
        text_rgb: (u8, u8, u8),
    ) -> Result<(), String> {
        // 转换为RGBA8格式（便于像素操作）
        let mut img_buf = image.to_rgba8();
        let (img_width, img_height) = (img_buf.width(), img_buf.height());

        // 计算文字边界（居中对齐）
        let v_metrics = font.v_metrics(scale);
        let glyphs: Vec<_> = font.layout(text, scale, point(0.0, 0.0)).collect();
        let text_width = glyphs.iter()
            .rev()
            .find_map(|g| g.pixel_bounding_box().map(|b| b.max.x))
            .unwrap_or(0) as f32;
        let text_height = (v_metrics.ascent - v_metrics.descent) as f32;

        // 最终文字位置（居中）
        let x = pos.0 - text_width / 2.0;
        let y = pos.1 + text_height / 2.0 - v_metrics.descent;

        // 逐字符绘制像素
        for glyph in font.layout(text, scale, point(x, y)) {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, alpha| {
                    let px = bounding_box.min.x + gx as i32;
                    let py = bounding_box.min.y + gy as i32;
                    // 确保像素在图片范围内
                    if px >= 0 && px < img_width as i32 && py >= 0 && py < img_height as i32 {
                        let pixel = img_buf.get_pixel_mut(px as u32, py as u32);
                        // 文字颜色混合（Alpha通道）
                        let alpha = alpha as f32;
                        pixel.0[0] = ((1.0 - alpha) * pixel.0[0] as f32 + alpha * text_rgb.0 as f32) as u8;
                        pixel.0[1] = ((1.0 - alpha) * pixel.0[1] as f32 + alpha * text_rgb.1 as f32) as u8;
                        pixel.0[2] = ((1.0 - alpha) * pixel.0[2] as f32 + alpha * text_rgb.2 as f32) as u8;
                    }
                });
            }
        }

        // 将修改后的缓冲区写回原图片
        *image = image::DynamicImage::ImageRgba8(img_buf);
        Ok(())
    }

    // ========== 完整的导出图片函数（包含文字绘制） ==========
    fn export_combined_image(&self) -> Result<(), String> {
        // 检查必要条件
        let selected_path = self.selected_file.as_ref()
            .ok_or("未选择图片文件")?;
        let original_image = self.original_image.as_ref()
            .ok_or("未加载原始图片数据")?;

        let original_width = original_image.width();
        let original_height = original_image.height();

        // 计算新图片尺寸（原图片尺寸 + 偏移值）
        let bg_width = original_width + self.left_value as u32 + self.right_value as u32;
        let bg_height = original_height + self.up_value as u32 + self.down_value as u32;

        // 创建匹配原图片位深的背景
        let mut bg_image: image::DynamicImage = match original_image {
            image::DynamicImage::ImageRgba8(_) => {
                let bg_buf = image::ImageBuffer::from_pixel(bg_width, bg_height, image::Rgba([
                    self.bg_color.r(), 
                    self.bg_color.g(), 
                    self.bg_color.b(), 
                    self.bg_color.a()
                ]));
                image::DynamicImage::ImageRgba8(bg_buf)
            }
            image::DynamicImage::ImageRgb8(_) => {
                let bg_buf = image::ImageBuffer::from_pixel(bg_width, bg_height, image::Rgb([
                    self.bg_color.r(), 
                    self.bg_color.g(), 
                    self.bg_color.b()
                ]));
                image::DynamicImage::ImageRgb8(bg_buf)
            }
            image::DynamicImage::ImageRgba16(_) => {
                let bg_buf = image::ImageBuffer::from_pixel(bg_width, bg_height, image::Rgba([
                    self.bg_color.r() as u16 * 257, 
                    self.bg_color.g() as u16 * 257, 
                    self.bg_color.b() as u16 * 257, 
                    self.bg_color.a() as u16 * 257
                ]));
                image::DynamicImage::ImageRgba16(bg_buf)
            }
            _ => {
                let bg_buf = image::ImageBuffer::from_pixel(bg_width, bg_height, image::Rgba([255, 255, 255, 255]));
                image::DynamicImage::ImageRgba8(bg_buf)
            }
        };

        // 将原图片按偏移位置叠加到背景（保留原始像素）
        let offset_x = self.left_value as i64;
        let offset_y = self.up_value as i64;
        image::imageops::overlay(
            &mut bg_image,
            original_image,
            offset_x as i64,
            offset_y as i64,
        );

        // 准备要绘制的EXIF文字信息
        let mut text_lines = Vec::new();
        if let Some(camera_model) = self.rows.iter().find(|r| r.tag_name == "相机型号") {
            text_lines.push(format!("{}", camera_model.tag_value));
        }
        if let Some(iso) = self.rows.iter().find(|r| r.tag_name == "ISO") {
            text_lines.push(format!("ISO{}", iso.tag_value));
        }
        if let Some(aperture) = self.rows.iter().find(|r| r.tag_name == "光圈") {
            text_lines.push(format!("{}", aperture.tag_value));
        }
        if let Some(exposure) = self.rows.iter().find(|r| r.tag_name == "曝光时长") {
            text_lines.push(format!("{}", exposure.tag_value));
        }
        if let Some(focal_length) = self.rows.iter().find(|r| r.tag_name == "焦距") {
            text_lines.push(format!("{}", focal_length.tag_value));
        }

        // 兜底：如果无EXIF字段，强制添加测试文字
        if text_lines.is_empty() {
            text_lines.push("测试文字1".to_string());
            text_lines.push("测试文字2".to_string());
            eprintln!("无EXIF字段，添加测试文字：{:?}", text_lines);
        }

        // 计算文字颜色（与背景形成对比）
        let (r, g, b, _) = self.bg_color.to_tuple();
        let brightness = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0;
        let text_rgb = if brightness > 0.5 {
            (0, 0, 0)  // 浅色背景用黑色文字
        } else {
            (255, 255, 255)  // 深色背景用白色文字
        };

        // 加载字体用于文字绘制（适配多系统）
        let font_data = self.load_system_font()?;
        let font = Font::try_from_bytes(&font_data)
            .ok_or_else(|| format!("无法加载字体: 无效字体数据"))?;

        // 计算文字区域高度和位置
        let text_area_height = (bg_height as f32 * 0.15) as u32; // 占背景高度的15%
        let text_start_y = offset_y + original_height as i64 + (self.down_value as i64 - text_area_height as i64) / 2;
        
        // 检查文字区域是否有效
        let (text_start_y, text_area_height) = if text_area_height == 0 || text_start_y < 0 {
            let new_text_y = (bg_height - 50) as i64;
            let new_text_h = 50;
            eprintln!("文字区域无效，强制调整：y={}, 高度={}", new_text_y, new_text_h);
            (new_text_y, new_text_h as u32)
        } else {
            (text_start_y, text_area_height)
        };
        
        // 计算字体大小（根据文字区域高度动态调整）
        let font_size = (text_area_height as f32 * 0.6) as f32;
        let scale = Scale::uniform(font_size);

        // 计算每行文字的位置并绘制
        let total_text_width = bg_width - 40; // 减去边距
        let text_spacing = if text_lines.len() > 0 {
            total_text_width / text_lines.len() as u32
        } else {
            0
        };
        
        for (i, text) in text_lines.iter().enumerate() {
            let x_pos = (offset_x + 20 + (i as i64 * text_spacing as i64) + text_spacing as i64 / 2) as f32;
            let y_pos = text_start_y as f32 + text_area_height as f32 / 2.0;
            
            eprintln!("绘制文字「{}」，坐标({},{})", text, x_pos, y_pos);
            self.draw_text_to_image(
                &mut bg_image,
                &font,
                scale,
                text,
                (x_pos, y_pos),
                text_rgb
            )?;
        }

        // 处理文件名（添加_exif_frame后缀）
        let original_path = std::path::PathBuf::from(selected_path);
        let original_filename = original_path.file_stem()
            .ok_or("无法获取文件名")?
            .to_str()
            .ok_or("文件名无效")?;
        let original_ext = original_path.extension()
            .ok_or("无法获取文件扩展名")?
            .to_str()
            .ok_or("扩展名无效")?
            .to_lowercase();

        // 弹出保存对话框
        let default_save_name = format!("{}_exif_frame.{}", original_filename, original_ext);
        let save_path = rfd::FileDialog::new()
            .set_file_name(&default_save_name)
            .add_filter("支持的格式", &["jpg", "jpeg", "png"])
            .save_file()
            .ok_or("用户取消保存")?;

        // 确定导出格式
        let export_format = match save_path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("png") => image::ImageFormat::Png,
            Some("jpg") | Some("jpeg") => image::ImageFormat::Jpeg,
            _ => match original_ext.as_str() {
                "png" => image::ImageFormat::Png,
                "jpg" | "jpeg" => image::ImageFormat::Jpeg,
                _ => image::ImageFormat::Png,
            },
        };

        // JPG兼容处理（JPG不支持16位）
        let final_bg_image = if export_format == image::ImageFormat::Jpeg {
            match &bg_image {
                image::DynamicImage::ImageRgba16(img) => {
                    image::DynamicImage::ImageRgba16(img.clone()).to_rgba8()
                },
                image::DynamicImage::ImageRgb16(img) => {
                    image::DynamicImage::ImageRgb16(img.clone()).to_rgba8()
                },
                _ => bg_image.to_rgba8(),
            }
        } else {
            bg_image.to_rgba8()
        };

        // 保存最终图片
        let mut output_file = File::create(&save_path)
            .map_err(|e| format!("无法创建输出文件: {}", e))?;
        
        final_bg_image.write_to(&mut output_file, export_format)
            .map_err(|e| format!("保存图片失败: {}", e))?;

        Ok(())
    }

    // ========== 新增：多系统字体加载函数 ==========
    fn load_system_font(&self) -> Result<Vec<u8>, String> {
        // 根据操作系统选择字体路径
        if cfg!(windows) {
            // Windows 系统字体（宋体/微软雅黑）
            let font_paths = [
                "C:\\Windows\\Fonts\\simsun.ttc",    // 宋体
                "C:\\Windows\\Fonts\\msyh.ttc",     // 微软雅黑
                "C:\\Windows\\Fonts\\arial.ttf"     // 备用英文字体
            ];
            for path in font_paths {
                if let Ok(data) = std::fs::read(path) {
                    eprintln!("加载Windows字体成功: {}", path);
                    return Ok(data);
                }
            }
            return Err("Windows系统未找到可用字体".to_string());
        } else if cfg!(linux) {
            // Linux 系统字体
            let font_paths = [
                "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"
            ];
            for path in font_paths {
                if let Ok(data) = std::fs::read(path) {
                    eprintln!("加载Linux字体成功: {}", path);
                    return Ok(data);
                }
            }
            return Err("Linux系统未找到可用字体".to_string());
        } else if cfg!(macos) {
            // macOS 系统字体
            let font_paths = [
                "/Library/Fonts/Arial.ttf",
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/Helvetica.ttf"
            ];
            for path in font_paths {
                if let Ok(data) = std::fs::read(path) {
                    eprintln!("加载macOS字体成功: {}", path);
                    return Ok(data);
                }
            }
            return Err("macOS系统未找到可用字体".to_string());
        } else {
            // 兜底：使用内置测试字体（需将字体文件放在项目根目录fonts文件夹）
            if let Ok(data) = std::fs::read("fonts/DejaVuSans.ttf") {
                eprintln!("加载内置字体成功");
                return Ok(data);
            }
            return Err("未找到任何可用字体，请确保fonts/DejaVuSans.ttf存在".to_string());
        }
    }

    fn reset(&mut self) {
        // 重置 EXIF 数据到初始状态
        self.rows = self.initial_exif_data.clone();
    }

    fn read_exif(&mut self, path: &str) {
        self.rows = self.default_rows.clone();
        let extension = Path::new(path).extension().and_then(|ext| ext.to_str()).map(|s| s.to_lowercase());

        match File::open(path) {
            Ok(file) => {
                match extension.as_deref() {
                    Some("jpg") | Some("jpeg") => {
                        let mut buf_reader = BufReader::new(&file);
                        match Reader::new().read_from_container(&mut buf_reader) {
                            Ok(exif) => {
                                for row in &mut self.rows {
                                    analyse_jpg(&exif, row);
                                }
                            }
                            Err(e) => {
                                eprintln!("无法读取JPEG EXIF数据: {}", e);
                                if let Some(first_row) = self.rows.first_mut() {
                                    first_row.tag_value = format!("无法读取EXIF数据: {}", e);
                                }
                            }
                        }
                    }
                    Some("png") => {
                        // 对于 PNG，只解析一次文件，然后使用数据更新所有行
                        if let Ok(mut file) = File::open(path) {
                            let mut buf_reader = BufReader::new(&mut file);
                            match get_png_exif(&mut buf_reader) {
                                Ok((exif_data, dimensions, xmp_exif_data)) => {
                                    // 将 Vec<u8> 包装为 Option<Vec<u8>>
                                    let exif_data_opt = if exif_data.is_empty() {
                                        None
                                    } else {
                                        Some(exif_data)
                                    };
                                    
                                    // 使用获取的数据更新所有行
                                    for row in &mut self.rows {
                                        analyse_png(row, &exif_data_opt, &dimensions, &xmp_exif_data);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("PNG EXIF extraction error: {}", e);
                                    for row in &mut self.rows {
                                        if row.tag_value.is_empty() {
                                            row.tag_value = "无EXIF数据".to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // 对于其他格式，显示不支持的信息
                        if let Some(first_row) = self.rows.first_mut() {
                            first_row.tag_value = "不支持该格式的EXIF读取".to_string();
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("无法打开文件: {}", e);
                if let Some(first_row) = self.rows.first_mut() {
                    first_row.tag_value = format!("无法打开文件: {}", e);
                }
            }
        }
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
            .default_width(available_width * 0.3)  // 默认宽度
            .min_width(available_width * 0.3)  // 最小宽度
            .max_width(available_width * 0.4)  // 最大宽度
            .show(ctx, |ui| {

                self.left_panel_width = ui.available_width();  // 更新实际宽度

                custom_text(ui, "EXIF 信息查看器", "heading", 
                Some(TextOptions { 
                    size: Some(20.0), 
                    color: Some(Vector4::new(180, 200, 150, 255)),
                    align: "CENTER"
                }));

                ui.hyperlink_to("EXIF Viewer", "https://exifviewer.com/zh/");

                ui.separator();

                // 添加滚动区域
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if ui.button("选择文件").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("图片文件", &["png", "jpg", "jpeg", "nef"])
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

                        ui.separator();
                        
                        // 检查是否有EXIF数据
                        let has_exif_data = self.rows.iter().any(|row| !row.tag_value.is_empty());
                        
                        // 改进空数据提示信息
                        if !has_exif_data {
                            ui.centered_and_justified(|ui| {
                                ui.label("暂无EXIF数据，请选择图片文件");
                                ui.label("支持的格式: PNG, JPG, JPEG");
                            });
                            return;
                        }

                        egui::Grid::new("exif_grid")
                            .num_columns(2)
                            .spacing([20.0, 8.0]) // 行、列间距
                            .striped(true) // 斑马纹
                            .min_col_width(40.0)
                            .show(ui, |ui| {
                                // 表头
                                ui.heading(RichText::new("标签名称").size(16.0));
                                ui.heading(RichText::new("值").size(16.0));
                                ui.end_row();

                                for (i, row) in self.rows.iter_mut().enumerate() { // 每行遍历添加
                                    // 第一列: 固定文本
                                    ui.label(RichText::new(&row.tag_name).size(14.0));
                                    // 第二列: 部分可编辑
                                    if row.tag_name == "图像宽度" || row.tag_name == "图像高度" {
                                        ui.label(RichText::new(&row.tag_value).size(14.0));
                                    } else {
                                        ui.text_edit_singleline(&mut row.tag_value);
                                    }
                                    ui.end_row();
                                }
                            }
                        );

                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label(format!("总行数: {}", self.rows.len()));
                            // 重置按钮 - 恢复到第一次加载图片时的EXIF数据
                            if ui.button("重置到初始数据").clicked() {
                                self.reset(); // 使用自定义重置方法
                            }
                            // 添加提示文本说明重置功能
                            ui.label(RichText::new("(恢复到第一次加载的数据)").small());
                        });

                        ui.separator();


                        // 主要的数值输入框
                        egui::Grid::new("bg_color_grid")
                            .num_columns(4)
                            .spacing([20.0, 10.0])
                            .show(ui, |ui| {

                            ui.label("上:");
                            ui.add(
                                egui::DragValue::new(&mut self.up_value)
                                    .range(self.min_value..=self.max_value)             // 设置数值范围
                                    .speed(self.step)                                   // 设置调整速度
                                    .fixed_decimals(self.decimal_places)  // 固定小数位数
                            );
                            ui.label("下:");
                            ui.add(
                                egui::DragValue::new(&mut self.down_value)
                                    .range(self.min_value..=self.max_value)
                                    .speed(self.step)
                                    .fixed_decimals(self.decimal_places)
                            );
                            ui.end_row();

                            ui.label("左:");
                            ui.add(
                                egui::DragValue::new(&mut self.left_value)
                                    .range(self.min_value..=self.max_value)
                                    .speed(self.step)
                                    .fixed_decimals(self.decimal_places)
                            );
                            ui.label("右:");
                            ui.add(
                                egui::DragValue::new(&mut self.right_value)
                                    .range(self.min_value..=self.max_value)
                                    .speed(self.step)
                                    .fixed_decimals(self.decimal_places)
                            );
                        });
                        
                        ui.separator();

                        egui::Grid::new("config_grid")
                            .num_columns(4)
                            .spacing([20.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("最小值:");
                                ui.add(egui::DragValue::new(&mut self.min_value).speed(0.1));
                                
                                ui.label("最大值:");
                                ui.add(egui::DragValue::new(&mut self.max_value).speed(1));
                                ui.end_row();
                                
                                ui.label("步长:");
                                ComboBox::from_label(".")
                                    .selected_text(format!("{:.1}", self.step)) // 显示当前选中的步长
                                    .width(80.0) // 下拉框宽度
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.step, 0.1, "0.1");
                                        ui.selectable_value(&mut self.step, 1.0, "1.0");
                                    });
                                
                                ui.label("小数位数:");
                                ui.add(egui::DragValue::new(&mut self.decimal_places).range(0..=2).speed(1));
                                ui.end_row();

                                ui.label("背景色：");
                                ComboBox::from_label("")
                                    .selected_text(format!("#{:02X}{:02X}{:02X}", self.bg_color.r(),self.bg_color.b(),self.bg_color.g())) // 显示当前选中的背景色
                                    .width(80.0) // 下拉框宽度
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_value(&mut self.bg_color, egui::Color32::WHITE, "白色").clicked() {
                                            self.show_custom_bg_color_picker = false;
                                        };
                                        if ui.selectable_value(&mut self.bg_color, egui::Color32::BLACK, "黑色").clicked() {
                                            self.show_custom_bg_color_picker = false;
                                        };
                                        if ui.selectable_label(false, "自定义").clicked() {
                                            self.show_custom_bg_color_picker = true;
                                        }
                                    });
                                if self.show_custom_bg_color_picker {
                                    ui.horizontal(|ui| {
                                        egui::color_picker::color_edit_button_srgba(
                                            ui,
                                            &mut self.bg_color, // 直接绑定到背景色，实时修改
                                            egui::color_picker::Alpha::Opaque, // 仅不透明颜色
                                        );
                                    });
                                }
                                ui.end_row();

                                ui.label("启用模糊背景:");
                                ui.checkbox(&mut self.enable_blur_bg, ""); // 复选框控制是否启用

                                ui.label("模糊强度:");
                                let mut blur_enabled = self.enable_blur_bg && self.texture.is_some();
                                let blur_response = ui.add_enabled(
                                    blur_enabled,
                                    egui::DragValue::new(&mut self.blur_strength)
                                        .range(0.1..=5.0) // 模糊强度范围
                                        .speed(0.1)
                                );
                                if blur_response.changed() {
                                    self.blur_strength = self.blur_strength.clamp(0.1, 5.0);
                                }
                                ui.end_row();

                                // 图片加载状态提示
                                if self.texture.is_none() {
                                    ui.label(egui::RichText::new("图片未加载").color(egui::Color32::RED));
                                } else if !self.enable_blur_bg {
                                    ui.label(egui::RichText::new("已加载图片").color(egui::Color32::GREEN));
                                } else {
                                    ui.label(egui::RichText::new("模糊生效中").color(egui::Color32::BLUE));
                                }
                                ui.end_row();
                            });

                        ui.separator();

                        if ui.button("导出图片").clicked() {
                            match self.export_combined_image() {
                                Ok(_) => {
                                    // 设置成功提示
                                    self.export_toast = Some("✅ 导出成功！".to_string());
                                    self.export_toast_is_success = true;
                                }
                                Err(e) => {
                                    eprintln!("导出失败: {}", e);
                                    // 设置失败提示
                                    self.export_toast = Some(format!("❌ 导出失败: {}", e));
                                    self.export_toast_is_success = false;
                                }
                            }
                        }
                    }
                );
            }
        );

        // 右侧固定面板
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                if let Some(texture) = &self.texture {
                    let total_height = available_height;
                    let info_height = total_height * 0.15;
                    let image_area_height = total_height - info_height;
                    
                    // 垂直布局，先显示标题和分隔线
                    ui.vertical(|ui| {
                        custom_text(ui, "图片预览:", "heading",
                        Some(TextOptions {
                            size: Some(24.0),
                            color: None,
                            align: "LEFT",
                        }));
                        ui.separator();
                        
                        // 图片显示区域 - 占用85%高度
                        ui.scope_builder(
                            egui::UiBuilder::new()
                                .max_rect(egui::Rect::from_min_size(
                                    ui.available_rect_before_wrap().min,
                                    egui::vec2(ui.available_width(), image_area_height)
                                )),  
                            |ui| {
                                ui.centered_and_justified(|ui| {

                                    let (img_width, img_height) = if self.image_size.x > self.image_size.y {
                                        let img_width = ui.available_width();
                                        let img_ratio = self.image_size.x / self.image_size.y;
                                        let img_height = img_width / img_ratio;
                                        (img_width, img_height)
                                    } else {
                                        let img_height = ui.available_height();
                                        let img_ratio = self.image_size.y / self.image_size.x;
                                        let img_width = img_height / img_ratio;
                                        (img_width, img_height)
                                    };

                                    // 保留你的背景大小逻辑：图片尺寸 + 偏移值
                                    let bg_width = img_width + self.left_value as f32 + self.right_value as f32;
                                    let bg_height = img_height + self.up_value as f32 + self.down_value as f32;
                                    let ori_bg_size = egui::Vec2::new(bg_width, bg_height);
                                    let ori_display_size = egui::Vec2::new(img_width, img_height);

                                    // 获取可用显示区域
                                    let max_available_width = ui.available_width();
                                    let max_available_height = ui.available_height();

                                    // 计算整体缩放比例：让背景+图片刚好适配可用区域，不超出
                                    // 缩放逻辑：取宽、高方向缩放比例的最小值（确保整体能完全放入可用区域）
                                    let scale_x = if ori_bg_size.x > 0.0 { max_available_width / ori_bg_size.x } else { 1.0 };
                                    let scale_y = if ori_bg_size.y > 0.0 { max_available_height / ori_bg_size.y } else { 1.0 };
                                    let scale_factor = scale_x.min(scale_y).min(1.0); // 不放大（scale<=1），只缩小

                                    // 应用缩放：背景、图片、偏移值同步缩放，保持相对位置不变
                                    let scaled_bg_size = ori_bg_size * scale_factor;
                                    let scaled_display_size = ori_display_size * scale_factor;
                                    let scaled_left = self.left_value as f32 * scale_factor; // 缩放后的水平偏移
                                    let scaled_up = self.up_value as f32 * scale_factor;   // 缩放后的垂直偏移

                                    
                                    // 分配背景区域
                                    let (bg_rect, _) = ui.allocate_exact_size(
                                        scaled_bg_size,
                                        egui::Sense::hover()
                                    );

                                    // 绘制背景框
                                    ui.painter().rect_filled(
                                        bg_rect,
                                        0.0,
                                        self.bg_color
                                    );

                                    // 计算图片位置：基于缩放后的偏移，确保图片始终在背景框内
                                    // 图片左上角 = 背景左上角 + 缩放后的偏移值
                                    let image_pos = egui::Pos2::new(
                                        bg_rect.min.x + scaled_left,
                                        bg_rect.min.y + scaled_up
                                    );
                                    let image_rect = egui::Rect::from_min_size(image_pos, scaled_display_size);

                                    // 绘制图片
                                    ui.painter().image(
                                        texture.id(),
                                        image_rect,
                                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                        egui::Color32::WHITE, // 色调
                                    );

                                    // ========== 新增：计算文字区域矩形 + 绘制文字 ==========
                                    // 文字区域矩形：宽度=scaled_bg_size.x，高度=scaled_bg_size.y - scaled_display_size.y
                                    // 位置：背景框内，图片下方的区域
                                    let text_rect = egui::Rect::from_min_max(
                                        // 左上角：背景左边界，图片底部（背景上边界 + 图片垂直偏移 + 图片高度）
                                        egui::Pos2::new(
                                            bg_rect.min.x, 
                                            bg_rect.min.y + scaled_up + scaled_display_size.y
                                        ),
                                        // 右下角：背景右边界，背景下边界
                                        egui::Pos2::new(
                                            bg_rect.min.x + scaled_bg_size.x, 
                                            bg_rect.min.y + scaled_bg_size.y
                                        )
                                    );

                                    // 1. 可选：绘制文字区域的边框（便于调试，可删除）
                                    ui.painter().rect_stroke(
                                        text_rect,
                                        egui::Rounding::ZERO, // 使用 egui::Rounding::ZERO 替代 Rounding::none()
                                        egui::Stroke::new(1.0f32, egui::Color32::GRAY),
                                        egui::StrokeKind::Outside
                                    );


                                    // 2. 准备要显示的EXIF文字（和导出逻辑保持一致）
                                    let mut text_lines = Vec::new();
                                    if let Some(camera_model) = self.rows.iter().find(|r| r.tag_name == "相机型号") {
                                        text_lines.push(format!("{}", camera_model.tag_value));
                                    }
                                    if let Some(iso) = self.rows.iter().find(|r| r.tag_name == "ISO") {
                                        text_lines.push(format!("ISO{}", iso.tag_value));
                                    }
                                    if let Some(aperture) = self.rows.iter().find(|r| r.tag_name == "光圈") {
                                        text_lines.push(format!("{}", aperture.tag_value));
                                    }
                                    if let Some(exposure) = self.rows.iter().find(|r| r.tag_name == "曝光时长") {
                                        text_lines.push(format!("{}", exposure.tag_value));
                                    }
                                    if let Some(focal_length) = self.rows.iter().find(|r| r.tag_name == "焦距") {
                                        text_lines.push(format!("{}", focal_length.tag_value));
                                    }

                                    // 兜底：无数据时显示提示
                                    if text_lines.is_empty() {
                                        text_lines.push("无EXIF信息".to_string());
                                    }

                                    // 3. 计算文字颜色（与背景对比）
                                    let (r, g, b, _) = self.bg_color.to_tuple();
                                    let brightness = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0;
                                    let text_color = if brightness > 0.5 {
                                        egui::Color32::BLACK
                                    } else {
                                        egui::Color32::WHITE
                                    };

                                    // 4. 绘制文字到预览区域
                                    let text_area_height = text_rect.height();
                                    let font_size = (text_area_height * 0.6).max(12.0); // 最小字号12
                                    let text_spacing = if text_lines.len() > 0 {
                                        text_rect.width() / text_lines.len() as f32
                                    } else {
                                        0.0
                                    };

                                    for (i, text) in text_lines.iter().enumerate() {
                                        // 文字居中定位
                                        let text_x = text_rect.min.x + (i as f32 * text_spacing) + text_spacing / 2.0;
                                        let text_y = text_rect.center().y;

                                        // 绘制文字
                                        ui.painter().text(
                                            egui::Pos2::new(text_x, text_y),
                                            egui::Align2::CENTER_CENTER,
                                            text,
                                            egui::FontId::new(font_size, egui::FontFamily::Proportional),
                                            text_color
                                        );
                                    }

                                    // 替代方案（简化模糊占位，保留开关逻辑）
                                    // if self.enable_blur_bg {
                                    //     // egui 0.33 需通过 glow 上下文直接操作，此处先保留开关，后续单独实现
                                    //     ui.painter().rect_filled(
                                    //         bg_rect,
                                    //         egui::Rounding::ZERO,
                                    //         egui::Color32::from_rgba_premultiplied(
                                    //             self.bg_color.r(),
                                    //             self.bg_color.g(),
                                    //             self.bg_color.b(),
                                    //             (self.bg_color.a() as f32 * 0.8) as u8 // 模拟模糊透明度
                                    //         )
                                    //     );
                                    // }

                                });
                            }
                        );

                        /// 在图片区域和信息区域之间添加分割线
                        ui.separator();
                        
                        // 信息显示区域 - 固定在底部15%
                        ui.scope_builder(
                            egui::UiBuilder::new()
                                .max_rect(egui::Rect::from_min_size(
                                    ui.available_rect_before_wrap().min,
                                    egui::vec2(ui.available_width(), info_height)
                                )),
                            |ui| {
                                // 使用垂直居中布局确保内容不被遮挡
                                ui.vertical_centered(|ui| {
                                    // 显示图片尺寸信息
                                    ui.horizontal(|ui| {
                                        custom_text(ui, "图片尺寸：", "label", {
                                            Some(TextOptions {
                                                size: Some(16.0),
                                                color: Some(Vector4::new(200, 200, 200, 255)),
                                                align: "LEFT"
                                            })
                                        });
                                        custom_text(ui, &format!("{} x {}", self.image_size.x, self.image_size.y), "label", {
                                            Some(TextOptions {
                                                size: Some(16.0),
                                                color: Some(Vector4::new(200, 200, 200, 255)),
                                                align: "LEFT"
                                            })
                                        });
                                    });
                                    
                                    // 显示文件路径信息
                                    ui.horizontal(|ui| {
                                        custom_text(ui, "文件路径：", "label", {
                                            Some(TextOptions {
                                                size: Some(16.0),
                                                color: Some(Vector4::new(200, 200, 200, 255)),
                                                align: "LEFT"
                                            })
                                        });
                                        if let Some(file) = &self.selected_file {
                                            // 如果路径太长，进行截断显示
                                            let display_path = if file.len() > 80 {
                                                format!("...{}", &file[file.len()-77..])
                                            } else {
                                                file.clone()
                                            };
                                            custom_text(ui, &display_path, "label", {
                                                Some(TextOptions {
                                                    size: Some(16.0),
                                                    color: Some(Vector4::new(200, 200, 200, 255)),
                                                    align: "LEFT"
                                                })
                                            });
                                        } else {
                                            custom_text(ui, "未选择文件", "label", {
                                                Some(TextOptions {
                                                    size: Some(16.0),
                                                    color: Some(Vector4::new(200, 200, 200, 255)),
                                                    align: "LEFT"
                                                })
                                            });
                                        }
                                    });
                                });
                            }
                        );
                    });
                } else {
                    // 无图片时显示提示
                    ui.centered_and_justified(|ui| {
                        custom_text(ui, "请选择图片文件", "heading",
                        Some(TextOptions {
                            size: Some(20.0),
                            color: Some(Vector4::new(255, 100, 100, 255)),
                            align: "CENTER"
                        }));
                        ui.label("支持格式: PNG、JPG、JPEG");
                    });
                }

                // ========== 导出提示弹窗（修复线程安全问题） ==========
                // 克隆一份字符串和状态，这样在 UI 闭包中修改 self 时不会与不可变借用冲突
                if let Some((toast_msg, is_success)) = self.export_toast.clone().zip(Some(self.export_toast_is_success)) {
                    let text_color = if is_success {
                        egui::Color32::from_rgb(0, 200, 0)
                    } else {
                        egui::Color32::from_rgb(255, 0, 0)
                    };

                    egui::Window::new("导出结果")
                        .title_bar(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                        .auto_sized()
                        .show(ctx, |ui| {
                            // 渲染提示内容
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&toast_msg).size(16.0).color(text_color));
                                if ui.button("×").clicked() {
                                    self.export_toast = None;
                                    self.toast_timer = None;
                                }
                            });

                            // 自动关闭（3秒）
                            if let Some(start_time) = self.toast_timer {
                                if start_time.elapsed() >= std::time::Duration::from_secs(3) {
                                    self.export_toast = None;
                                    self.toast_timer = None;
                                }
                            }
                            ctx.request_repaint();
                        });
                }
                // 注意：不要在每帧都无条件重置导出提示和计时器，这会覆盖实际触发状态。
            });
    }
}

// ========== 主函数 ==========
fn main() -> Result<(), eframe::Error> {
    // 初始化原生后端（解决字体/文件对话框兼容）
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]), // 替代 initial_window_size
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..eframe::NativeOptions::default()
    };

    // 启动应用
    eframe::run_native(
        "EXIF图片编辑器",
        native_options,
        Box::new(|cc| {
            // 设置字体和样式
            setup_fonts_and_style(&cc.egui_ctx);
            
            // 创建应用实例
            Ok(Box::new(MyEguiApp::default()))
        }),
    )
}