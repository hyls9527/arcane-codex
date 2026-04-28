#!/usr/bin/env python3
"""Generate Windows ICO file from PNG"""
import struct
from PIL import Image
import io

def create_ico(png_path, ico_path):
    """Create multi-size ICO file from PNG"""
    # Open source image
    img = Image.open(png_path)
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    
    # Sizes for ICO file
    sizes = [16, 32, 48, 64, 128, 256]
    
    # ICO header
    ico_data = io.BytesIO()
    
    # ICO Header: Reserved (2 bytes), Type (2 bytes), Count (2 bytes)
    ico_data.write(struct.pack('<HHH', 0, 1, len(sizes)))
    
    # Calculate offset for image data
    offset = 6 + (len(sizes) * 16)  # Header + ICONDIRENTRY array
    
    # Store image data temporarily
    image_data_list = []
    
    for size in sizes:
        # Resize image
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        
        # Save as PNG to bytes
        img_bytes = io.BytesIO()
        resized.save(img_bytes, format='PNG')
        data = img_bytes.getvalue()
        image_data_list.append(data)
        
        # ICONDIRENTRY: Width, Height, Colors, Reserved, Planes, BitCount, Size, Offset
        width = size if size < 256 else 0  # 0 means 256
        height = size if size < 256 else 0
        ico_data.write(struct.pack('<BBBBHHII', 
            width, height, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
    
    # Write image data
    for data in image_data_list:
        ico_data.write(data)
    
    # Save ICO file
    with open(ico_path, 'wb') as f:
        f.write(ico_data.getvalue())
    
    print(f"Created {ico_path} with sizes: {sizes}")

if __name__ == '__main__':
    create_ico('256x256.png', 'icon.ico')
