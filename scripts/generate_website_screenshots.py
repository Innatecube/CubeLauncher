"""
CubeLauncher Automated Website Screenshot Generator
Navigates through all primary pages in the live running CubeLauncher application,
captures high-resolution DirectX/GPUI window buffers via Windows DWM PrintWindow,
and automatically updates website/images/*.png with pristine 1280x720-class native captures (zero taskbar).
"""

import os
import sys
import time
import ctypes
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from capture_cubelauncher import find_cubelauncher_window, capture_window

user32 = ctypes.windll.user32

WM_LBUTTONDOWN = 0x0201
WM_LBUTTONUP = 0x0202
WM_MOUSEMOVE = 0x0200

def make_lparam(x, y):
    return (y << 16) | (x & 0xffff)

def click(hwnd, x, y, delay=0.6):
    lp = make_lparam(x, y)
    user32.PostMessageW(hwnd, WM_MOUSEMOVE, 0, lp)
    time.sleep(0.04)
    user32.PostMessageW(hwnd, WM_LBUTTONDOWN, 1, lp)
    time.sleep(0.04)
    user32.PostMessageW(hwnd, WM_LBUTTONUP, 0, lp)
    time.sleep(delay)

def generate_all():
    hwnd = find_cubelauncher_window()
    if not hwnd:
        raise RuntimeError("CubeLauncher is not running!")

    print(f"Connected to CubeLauncher window: {hex(hwnd)}")
    dest_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "website", "images")
    os.makedirs(dest_dir, exist_ok=True)

    # 1. Instances Library Overview
    print("\n[1/5] Navigating to Instances Library (x=80, y=100)...")
    click(hwnd, 80, 100, delay=0.8)
    inst_path = os.path.join(dest_dir, "instance_client.png")
    capture_window(hwnd, inst_path)

    # 2. Discover Content Browser
    print("\n[2/5] Navigating to Discover Content (x=80, y=140)...")
    click(hwnd, 80, 140, delay=1.0)
    disc_path = os.path.join(dest_dir, "discover_client.png")
    capture_window(hwnd, disc_path)

    # 3. Hero Preview: Luxium Instance Detail (Fabric 1.20.1)
    print("\n[3/5] Navigating to Luxium Instance Detail (x=80, y=300)...")
    click(hwnd, 80, 300, delay=0.8)
    hero_path = os.path.join(dest_dir, "hero_client.png")
    capture_window(hwnd, hero_path)

    # 4. Mod Management: Create+ Instance Detail (NeoForge 1.21.1)
    print("\n[4/5] Navigating to Create+ Instance Detail (x=80, y=345)...")
    click(hwnd, 80, 345, delay=0.8)
    mods_path = os.path.join(dest_dir, "mods_client.png")
    capture_window(hwnd, mods_path)

    # 5. 3D Wardrobe & Skin Studio
    print("\n[5/5] Navigating to 3D Wardrobe (x=80, y=625)...")
    click(hwnd, 80, 625, delay=0.8)
    ward_path = os.path.join(dest_dir, "wardrobe_client.png")
    capture_window(hwnd, ward_path)

    print("\nAll website screenshots successfully generated and updated!")

if __name__ == "__main__":
    generate_all()
