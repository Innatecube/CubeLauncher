"""
CubeLauncher Automated Screenshot Capture Utility
Captures the live CubeLauncher client window cleanly without the Windows taskbar.
Uses native Windows DWM PrintWindow API (PW_RENDERFULLCONTENT) for hardware-accelerated,
lossless window buffer extraction.
"""

import sys
import os
import argparse
import ctypes
from ctypes import wintypes
from PIL import Image

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32

PW_RENDERFULLCONTENT = 2
SW_RESTORE = 9
GENERIC_ALL = 0x10000000

class BITMAPINFOHEADER(ctypes.Structure):
    _fields_ = [
        ('biSize', wintypes.DWORD),
        ('biWidth', wintypes.LONG),
        ('biHeight', wintypes.LONG),
        ('biPlanes', wintypes.WORD),
        ('biBitCount', wintypes.WORD),
        ('biCompression', wintypes.DWORD),
        ('biSizeImage', wintypes.DWORD),
        ('biXPelsPerMeter', wintypes.LONG),
        ('biYPelsPerMeter', wintypes.LONG),
        ('biClrUsed', wintypes.DWORD),
        ('biClrImportant', wintypes.DWORD),
    ]

def switch_to_default_desktop():
    try:
        hdesk = user32.OpenDesktopW('default', 0, False, GENERIC_ALL)
        if hdesk:
            user32.SetThreadDesktop(hdesk)
    except Exception as e:
        print(f"Warning: could not switch desktop: {e}")

def find_cubelauncher_window():
    switch_to_default_desktop()
    hwnd = user32.FindWindowW(None, "CubeLauncher")
    if not hwnd:
        # Fallback enum
        def callback(h, extra):
            length = user32.GetWindowTextLengthW(h)
            buf = ctypes.create_unicode_buffer(length + 1)
            user32.GetWindowTextW(h, buf, length + 1)
            if "CubeLauncher" in buf.value:
                extra.append(h)
            return True
        found = []
        cb = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)(callback)
        user32.EnumWindows(cb, ctypes.byref(ctypes.py_object(found)))
        if found:
            hwnd = found[0]
    return hwnd

def capture_window(hwnd, output_path, crop_margins=True):
    if not hwnd:
        raise RuntimeError("CubeLauncher window not found! Ensure CubeLauncher is running.")

    if user32.IsIconic(hwnd):
        user32.ShowWindow(hwnd, SW_RESTORE)

    rect = wintypes.RECT()
    user32.GetWindowRect(hwnd, ctypes.byref(rect))
    w = rect.right - rect.left
    h = rect.bottom - rect.top

    if w <= 0 or h <= 0:
        raise RuntimeError(f"Invalid window dimensions: {w}x{h}")

    hdc_window = user32.GetWindowDC(hwnd)
    hdc_mem = gdi32.CreateCompatibleDC(hdc_window)
    hbm = gdi32.CreateCompatibleBitmap(hdc_window, w, h)
    gdi32.SelectObject(hdc_mem, hbm)

    success = user32.PrintWindow(hwnd, hdc_mem, PW_RENDERFULLCONTENT)
    if not success:
        gdi32.DeleteObject(hbm)
        gdi32.DeleteDC(hdc_mem)
        user32.ReleaseDC(hwnd, hdc_window)
        raise RuntimeError("PrintWindow failed to capture CubeLauncher buffer.")

    bmi = BITMAPINFOHEADER()
    bmi.biSize = ctypes.sizeof(BITMAPINFOHEADER)
    bmi.biWidth = w
    bmi.biHeight = -h # top-down
    bmi.biPlanes = 1
    bmi.biBitCount = 32
    bmi.biCompression = 0

    buf = ctypes.create_string_buffer(w * h * 4)
    gdi32.GetDIBits(hdc_mem, hbm, 0, h, buf, ctypes.byref(bmi), 0)

    gdi32.DeleteObject(hbm)
    gdi32.DeleteDC(hdc_mem)
    user32.ReleaseDC(hwnd, hdc_window)

    img = Image.frombuffer('RGBA', (w, h), buf, 'raw', 'BGRA', 0, 1).convert('RGB')

    # Trim maximized OS shadow margins (-7px left, top, right, bottom)
    if crop_margins and rect.left < 0 and rect.top < 0:
        # Maximized window on Windows 11
        crop_box = (8, 8, w - 8, h - 8)
        img = img.crop(crop_box)

    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    img.save(output_path, quality=95, optimize=True)
    print(f"Captured {output_path} ({img.size[0]}x{img.size[1]}) successfully without taskbar!")
    return img

def main():
    parser = argparse.ArgumentParser(description="Capture CubeLauncher window without Windows taskbar.")
    parser.add_argument("--output", "-o", default="website/images/hero_client.png", help="Output PNG path")
    args = parser.parse_args()

    hwnd = find_cubelauncher_window()
    if not hwnd:
        print("Error: CubeLauncher is not running.")
        sys.exit(1)

    print(f"Found CubeLauncher HWND: {hex(hwnd)}")
    capture_window(hwnd, args.output)

if __name__ == "__main__":
    main()
