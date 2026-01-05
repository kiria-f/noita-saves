import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
VENV_DIR = HERE / '.venv'
PYTHON_BIN = VENV_DIR / 'bin' / 'python' if os.name != 'nt' else VENV_DIR / 'Scripts' / 'python.exe'
PYTHONPATH = HERE
SCRIPT = HERE / 'app' / 'app.py'


def clear_screen():
    command = 'cls' if os.name == 'nt' else 'clear'
    os.system(command)


def create_venv():
    clear_screen()
    print('\n📦 Создаём виртуальное окружение...\n')
    subprocess.check_call([sys.executable, '-m', 'venv', str(VENV_DIR)])


def install_requirements():
    clear_screen()
    print('\n📥 Устанавливаем зависимости...\n')
    subprocess.check_call([str(PYTHON_BIN), '-m', 'pip', 'install', '--upgrade', 'pip'])
    subprocess.check_call([str(PYTHON_BIN), '-m', 'pip', 'install', '-r', 'requirements.txt'])


def main():
    if not PYTHON_BIN.exists():
        create_venv()
        install_requirements()

    env = os.environ.copy()
    env['PYTHONPATH'] = str(PYTHONPATH)
    env['APP_ROOT_DIR'] = str(HERE)

    clear_screen()
    os.execve(str(PYTHON_BIN), [str(PYTHON_BIN), str(SCRIPT)], env)


if __name__ == '__main__':
    main()
