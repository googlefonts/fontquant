import tempfile
import os
import time
import threading
from reportlab.graphics import renderPM
from PIL import Image
from svglib.svglib import svg2rlg


def svg_to_img(svg):
    current_time = time.time()
    temp_folder = tempfile.gettempdir()
    svg_path = os.path.join(temp_folder, f"svg_to_png.{current_time}.{os.getpid()}.{threading.get_ident()}.svg")
    with open(svg_path, "w") as f:
        f.write(svg)

    # Convert to PNG
    png_path = os.path.join(temp_folder, f"svg_to_png.{current_time}.{os.getpid()}.{threading.get_ident()}.png")

    drawing = svg2rlg(svg_path)
    renderPM.drawToFile(drawing, png_path, fmt="PNG")

    img = Image.open(png_path).convert("RGB")

    # Remove files
    os.remove(svg_path)
    os.remove(png_path)

    return img
