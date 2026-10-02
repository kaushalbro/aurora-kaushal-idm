#!/usr/bin/env python3
import os
from PIL import Image, ImageDraw

def render_icon(target_size):
    scale = 4
    size = target_size * scale
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Outer squircle filling 100% of the canvas
    radius = int(size * 0.22)
    bg_color = (0, 122, 255, 255) # Solid Apple Blue #007AFF
    draw.rounded_rectangle([(0, 0), (size - 1, size - 1)], radius=radius, fill=bg_color)
    
    cx = size / 2.0
    
    # Large, bold download arrow + bucket
    stem_half_w = size * 0.10
    head_half_w = size * 0.28
    
    arrow_top = size * 0.15
    arrow_neck = size * 0.44
    arrow_tip = size * 0.68
    
    white = (255, 255, 255, 255)
    
    # Arrow polygon
    p_tip = (cx, arrow_tip)
    p_wing_l = (cx - head_half_w, arrow_neck)
    p_inner_l = (cx - stem_half_w, arrow_neck)
    p_top_l = (cx - stem_half_w, arrow_top)
    p_top_r = (cx + stem_half_w, arrow_top)
    p_inner_r = (cx + stem_half_w, arrow_neck)
    p_wing_r = (cx + head_half_w, arrow_neck)
    
    draw.polygon([p_tip, p_wing_l, p_inner_l, p_top_l, p_top_r, p_inner_r, p_wing_r], fill=white)
    
    # Bucket / Tray at bottom
    bucket_w = size * 0.32
    bucket_bottom = size * 0.85
    bucket_h = size * 0.08
    bucket_lip_h = size * 0.13
    
    # Bucket base
    draw.rounded_rectangle(
        [(cx - bucket_w, bucket_bottom - bucket_h), (cx + bucket_w, bucket_bottom)],
        radius=max(1, int(size * 0.03)),
        fill=white
    )
    # Left lip
    draw.rounded_rectangle(
        [(cx - bucket_w, bucket_bottom - bucket_lip_h), (cx - bucket_w + bucket_h, bucket_bottom)],
        radius=max(1, int(size * 0.03)),
        fill=white
    )
    # Right lip
    draw.rounded_rectangle(
        [(cx + bucket_w - bucket_h, bucket_bottom - bucket_lip_h), (cx + bucket_w, bucket_bottom)],
        radius=max(1, int(size * 0.03)),
        fill=white
    )
    
    # Resample to target size with Lanczos for super-crisp antialiasing
    final_img = img.resize((target_size, target_size), Image.Resampling.LANCZOS)
    return final_img

def main():
    ext_icons_dir = os.path.join(os.path.dirname(__file__), "..", "apps", "aurora-extension", "icons")
    gui_res_dir = os.path.join(os.path.dirname(__file__), "..", "apps", "aurora-gui", "resources")
    os.makedirs(ext_icons_dir, exist_ok=True)
    os.makedirs(gui_res_dir, exist_ok=True)
    
    images_for_ico = []
    sizes = [16, 24, 32, 48, 64, 128, 256, 512]
    
    for size in sizes:
        icon = render_icon(size)
        out_ext = os.path.join(ext_icons_dir, f"icon-{size}.png")
        icon.save(out_ext, "PNG")
        
        out_gui = os.path.join(gui_res_dir, f"icon-{size}.png")
        icon.save(out_gui, "PNG")
        
        if size <= 256:
            images_for_ico.append(icon)
        print(f"Generated icon-{size}.png ({size}x{size})")

    # Generate multi-layer Windows ICO
    ico_path = os.path.join(gui_res_dir, "icon.ico")
    images_for_ico[-1].save(
        ico_path,
        format="ICO",
        sizes=[(img.width, img.height) for img in images_for_ico],
        append_images=images_for_ico[:-1]
    )
    print(f"Generated multi-resolution Windows icon.ico ({ico_path})")

if __name__ == "__main__":
    main()


