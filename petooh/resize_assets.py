import os
from PIL import Image

def resize_for_bonanza(target_size=(512, 512), threshold=240):
    # Создаем папку для готовых файлов, если её нет
    output_dir = "ready_assets"
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    # Список расширений, которые будем обрабатывать
    valid_extensions = ('.png', '.jpg', '.jpeg', '.webp')

    print(f"🚀 Начинаю обработку изображений в {target_size}...")

    for filename in os.listdir('.'):
        if filename.lower().endswith(valid_extensions):
            try:
                # Открываем изображение и конвертируем в RGBA (для прозрачности)
                with Image.open(filename) as img:
                    img = img.convert("RGBA")
                    
                    # 1. Удаление белого фона (превращаем белый в прозрачный)
                    data = img.getdata()
                    new_data = []
                    for item in data:
                        # Если пиксель очень светлый (близко к 255, 255, 255)
                        if item[0] > threshold and item[1] > threshold and item[2] > threshold:
                            new_data.append((255, 255, 255, 0)) # Полная прозрачность
                        else:
                            new_data.append(item)
                    img.putdata(new_data)

                    # 2. Обрезка пустых краев (чтобы конфета была по размеру объекта)
                    bbox = img.getbbox()
                    if bbox:
                        img = img.crop(bbox)

                    # 3. Масштабирование с сохранением пропорций
                    img.thumbnail(target_size, Image.Resampling.LANCZOS)

                    # 4. Создание финального квадратного полотна и центрирование
                    final_card = Image.new("RGBA", target_size, (0, 0, 0, 0))
                    offset = (
                        (target_size[0] - img.size[0]) // 2,
                        (target_size[1] - img.size[1]) // 2
                    )
                    final_card.paste(img, offset)

                    # Сохраняем результат
                    clean_name = os.path.splitext(filename)[0]
                    final_path = os.path.join(output_dir, f"{clean_name}.png")
                    final_card.save(final_path, "PNG")
                    print(f"✅ Готово: {final_path}")

            except Exception as e:
                print(f"❌ Ошибка в файле {filename}: {e}")

    print("\n✨ Все изображения в папке 'ready_assets'!")

if __name__ == "__main__":
    resize_for_bonanza()
