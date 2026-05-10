import os
import sys

# Расширения файлов, которые заведомо бинарные и не нужны в выводе
BINARY_EXTENSIONS = {'.o', '.rlib', '.d', '.so', '.dylib', '.dll', '.exe',
                     '.bin', '.obj', '.pdb', '.wasm', '.gz', '.zip', '.tar',
                     '.db', '.sqlite', '.pyc', '.pyo', '.parquet', '.arrow'}

def should_exclude_dir(dirname):
    """Исключаемые каталоги (не заходить внутрь)"""
    return dirname in ("target", ".git", "node_modules", "__pycache__", "debug", "release")

def should_exclude_file(filename):
    """Исключаемые файлы по имени или расширению"""
    if filename in ("Cargo.lock", "Cargo.toml.bak"):
        return True
    ext = os.path.splitext(filename)[1].lower()
    return ext in BINARY_EXTENSIONS

def write_directory_contents_to_file(root_dir, output_file, encoding='utf-8'):
    with open(output_file, 'w', encoding='utf-8') as out_f:
        for current_dir, subdirs, files in os.walk(root_dir):
            # 1. Проверяем имя текущего каталога (не полный путь)
            dirname = os.path.basename(current_dir)
            if should_exclude_dir(dirname):
                # Очищаем список subdirs, чтобы os.walk не заходил внутрь
                subdirs.clear()
                continue

            for file in files:
                if should_exclude_file(file):
                    continue

                file_path = os.path.join(current_dir, file)
                out_f.write(f"# {file_path}\n")

                # 2. Дополнительная защита от больших файлов (например, логи)
                try:
                    size = os.path.getsize(file_path)
                    if size > 50 * 1024 * 1024:  # 50 MB
                        out_f.write(f"[Файл {size//1024//1024} МБ, пропущен]\n")
                    else:
                        with open(file_path, 'r', encoding=encoding, errors='replace') as in_f:
                            content = in_f.read()
                            out_f.write(content)
                except Exception as e:
                    out_f.write(f"[Ошибка чтения файла: {e}]\n")
                out_f.write("\n\n")

def main():
    if len(sys.argv) == 1:
        print("Использование:")
        print("  python3 test.py <выходной_файл>            # обход текущей папки")
        print("  python3 test.py <каталог> <выходной_файл>  # обход указанной папки")
        sys.exit(1)

    if len(sys.argv) == 2:
        root_dir = os.getcwd()
        output_file = sys.argv[1]
    else:
        root_dir = sys.argv[1]
        output_file = sys.argv[2]
        if not os.path.isdir(root_dir):
            print(f"Ошибка: '{root_dir}' не является каталогом.")
            sys.exit(1)

    print(f"Обход каталога: {root_dir}")
    print(f"Исключаются каталоги: target, .git, __pycache__, debug, release")
    print(f"Исключаются бинарные расширения: {', '.join(sorted(BINARY_EXTENSIONS))}")
    print(f"Запись результата в: {output_file}")
    write_directory_contents_to_file(root_dir, output_file)
    print(f"Готово. Файл создан: {output_file}")

if __name__ == "__main__":
    main()