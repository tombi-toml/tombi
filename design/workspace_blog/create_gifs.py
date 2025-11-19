#!/usr/bin/env python3
"""Create GIF animations from before/after screenshot pairs with labels."""

from PIL import Image, ImageDraw, ImageFont
import sys
from pathlib import Path

def add_label_to_image(img: Image.Image, label: str, font_size: int = 80) -> Image.Image:
    """
    Add a bold red label to the top-left corner of an image.

    Args:
        img: The image to add label to
        label: The text label (e.g., "Before", "After")
        font_size: Font size for the label (default: 80)

    Returns:
        A new image with the label added
    """
    # Create a copy to avoid modifying the original
    labeled_img = img.copy()
    draw = ImageDraw.Draw(labeled_img)

    # Try to use a bold font
    try:
        # Try common bold system fonts (macOS, Linux, Windows)
        font_paths = [
            "/System/Library/Fonts/Helvetica.ttc",  # macOS Helvetica Bold
            "/System/Library/Fonts/SFNSDisplay.ttf",  # macOS San Francisco
            "/Library/Fonts/Arial Bold.ttf",  # macOS Arial Bold
            "/System/Library/Fonts/Supplemental/Arial Bold.ttf",  # macOS Arial Bold
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",  # Linux
            "C:\\Windows\\Fonts\\arialbd.ttf",  # Windows Arial Bold
            "C:\\Windows\\Fonts\\ariblk.ttf",  # Windows Arial Black
        ]

        font = None
        for font_path in font_paths:
            try:
                font = ImageFont.truetype(font_path, font_size)
                break
            except:
                continue

        if font is None:
            # Try to load default font with larger size
            try:
                font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", font_size)
            except:
                # Last resort: default font
                font = ImageFont.load_default()
    except:
        font = ImageFont.load_default()

    # Position: adjusted position (up 20px, right 50px from original)
    padding_top = 10  # 30 - 20 = 10px from top
    padding_left = 180  # 130 + 50 = 180px from left
    x, y = padding_left, padding_top

    # Draw text in bold red with no background
    # Red color: RGB(220, 38, 38) - a vibrant red
    draw.text((x, y), label, fill=(220, 38, 38), font=font)

    return labeled_img

def create_gif(before_path: Path, after_path: Path, output_path: Path, duration: int = 1500):
    """
    Create a GIF animation from before and after images with labels.

    Args:
        before_path: Path to the "before" image
        after_path: Path to the "after" image
        output_path: Path where the GIF will be saved
        duration: Duration each frame is displayed in milliseconds (default: 1500ms)
    """
    try:
        # Open images
        before_img = Image.open(before_path)
        after_img = Image.open(after_path)

        # Add labels
        before_labeled = add_label_to_image(before_img, "Before")
        after_labeled = add_label_to_image(after_img, "After")

        # Create GIF
        before_labeled.save(
            output_path,
            save_all=True,
            append_images=[after_labeled],
            duration=duration,
            loop=0  # Loop forever
        )

        print(f"✓ Created: {output_path.name}")

    except Exception as e:
        print(f"✗ Error creating {output_path.name}: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return False

    return True

def main():
    # Use relative paths from script location
    script_dir = Path(__file__).parent
    source_dir = Path("/Users/s23467/develop/tombi/docs/public")
    output_dir = script_dir / "images"

    # Create output directory if it doesn't exist
    output_dir.mkdir(exist_ok=True)

    # Define image pairs: (before, after, output_name)
    image_pairs = [
        # uv CodeActions
        (
            "uv_CodeAction_UseWorkspaceDependency_Before.png",
            "uv_CodeAction_UseWorkspaceDependency_After.png",
            "uv_CodeAction_UseWorkspaceDependency.gif"
        ),
        (
            "uv_CodeAction_AddToWorkspaceAndUseWorkspaceDependency_Before.png",
            "uv_CodeAction_AddToWorkspaceAndUseWorkspaceDependency_After.png",
            "uv_CodeAction_AddToWorkspaceAndUseWorkspaceDependency.gif"
        ),
        # Cargo CodeActions
        (
            "Cargo_CodeAction_InheritFromWorkspace_Before.png",
            "Cargo_CodeAction_InheritFromWorkspace_After.png",
            "Cargo_CodeAction_InheritFromWorkspace.gif"
        ),
        (
            "Cargo_CodeAction_InheritDependencyFromWorkspace_Before.png",
            "Cargo_CodeAction_InheritDependencyFromWorkspace_After.png",
            "Cargo_CodeAction_InheritDependencyFromWorkspace.gif"
        ),
        (
            "Cargo_CodeAction_ConevrtDependencyToTableFormat_Before.png",
            "Cargo_CodeAction_ConevrtDependencyToTableFormat_After.png",
            "Cargo_CodeAction_ConvertDependencyToTableFormat.gif"
        ),
        (
            "Cargo_CodeAction_AddToWorkspaceAndInheritDependency_Before.png",
            "Cargo_CodeAction_AddToWorkspaceAndInheritDependency_After.png",
            "Cargo_CodeAction_AddToWorkspaceAndInheritDependency.gif"
        ),
    ]

    print(f"Creating GIF animations...")
    print(f"Source: {source_dir}")
    print(f"Output: {output_dir}")
    print()

    success_count = 0
    for before, after, output in image_pairs:
        before_path = source_dir / before
        after_path = source_dir / after
        output_path = output_dir / output

        if not before_path.exists():
            print(f"✗ Before image not found: {before}")
            continue

        if not after_path.exists():
            print(f"✗ After image not found: {after}")
            continue

        if create_gif(before_path, after_path, output_path):
            success_count += 1

    print()
    print(f"Successfully created {success_count}/{len(image_pairs)} GIF files.")

if __name__ == "__main__":
    main()
