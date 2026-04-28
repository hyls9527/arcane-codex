#!/usr/bin/env python3
"""Crop the 512x512 icon from the showcase image"""
from PIL import Image

# Open the showcase image
img = Image.open("D:/Personal/Documents/图标/成品-1.png")
width, height = img.size
print(f"Image size: {width}x{height}")

# The 512x512 icon is at the bottom center
# Based on the layout: 32x32 (top left), 128x128 (top middle), 256x256 (top right), 512x512 (bottom)
# The 512x512 icon appears to be centered at the bottom

# Crop region for 512x512 icon (estimated coordinates)
# The icon is roughly from x=156 to x=668, y=400 to y=912
left = 156
top = 400
right = 668
bottom = 912

cropped = img.crop((left, top, right, bottom))
cropped.save("D:/Personal/Documents/图标/icon_512_cropped.png")
print(f"Cropped icon saved: {cropped.size}")

# Also save as the main icon files
# Resize to various sizes for the app
sizes = [16, 32, 48, 64, 128, 256, 512]
for size in sizes:
    resized = cropped.resize((size, size), Image.Resampling.LANCZOS)
    resized.save(f"e:/knowledge base/src-tauri/icons/{size}x{size}.png")
    print(f"Saved {size}x{size}.png")

# Create ICO file
import struct
import io

def create_ico_from_png(png_path, ico_path):
    img = Image.open(png_path)
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    
    sizes = [16, 32, 48, 64, 128, 256]
    ico_data = io.BytesIO()
    
    # ICO Header
    ico_data.write(struct.pack('<HHH', 0, 1, len(sizes)))
    
    offset = 6 + (len(sizes) * 16)
    image_data_list = []
    
    for size in sizes:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        img_bytes = io.BytesIO()
        resized.save(img_bytes, format='PNG')
        data = img_bytes.getvalue()
        image_data_list.append(data)
        
        width_byte = size if size < 256 else 0
        height_byte = size if size < 256 else 0
        ico_data.write(struct.pack('<BBBBHHII', 
            width_byte, height_byte, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
    
    for data in image_data_list:
        ico_data.write(data)
    
    with open(ico_path, 'wb') as f:
        f.write(ico_data.getvalue())
    
    print(f"Created {ico_path}")

create_ico_from_png("D:/Personal/Documents/图标/icon_512_cropped.png", 
                    "e:/knowledge base/src-tauri/icons/icon.ico")
print("Done!")
