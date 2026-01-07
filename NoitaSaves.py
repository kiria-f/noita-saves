import importlib as imp
import os
import re
import shutil
import sys
import threading
import time
from filecmp import dircmp

CLEAR_SCREEN = False

FILENAME = str(os.path.basename(__file__).removesuffix('.py'))
INFO_FILE_NAME = '.noita_saves_info'
APPDATA = os.getenv('APPDATA')
APPDATA = APPDATA.replace('Roaming', 'LocalLow')
GAME_DIR = APPDATA + r'\Nolla_Games_Noita'
SAVES_DIR = APPDATA + r'\Nolla_Games_Noita_Saves'
CURRENT_SAVE = GAME_DIR + r'\save00'

PREV_LINE = '\033[1G\033[1A\033[0J'


def calc_folder_size(path: str) -> int:
    total_size = 0
    for dirPath, dirNames, filenames in os.walk(path):
        for f in filenames:
            fp = os.path.join(dirPath, f)
            if not os.path.islink(fp):
                total_size += os.path.getsize(fp)
    return total_size


def calc_folder_containment(path: str) -> int:
    total_count = 0
    for dirPath, dirNames, filenames in os.walk(path):
        total_count += len(filenames)
    return total_count


def get_folder_size_from_info_or_recalc_new(path: str) -> int:
    if INFO_FILE_NAME in os.listdir(path):
        with open(path + '\\' + INFO_FILE_NAME, 'r') as info_file:
            return int(info_file.readline())
    else:
        save_size = calc_folder_size(path)
        with open(path + '\\' + INFO_FILE_NAME, 'w') as info_file:
            info_file.write(str(save_size))
        return save_size


def print_progress(ratio):
    bar_len = 20
    filled_len = int(bar_len * ratio + 0.5)
    print(PREV_LINE, end='')
    print('[' + '#' * filled_len + ' ' * (bar_len - filled_len) + '] ' + str(int(ratio * 100 + 0.5)) + '%')


def run_with_progress(task, ratio_delegate):
    print()
    thread = threading.Thread(target=task)
    thread.start()
    ratio = ratio_delegate()
    print_progress(ratio)
    while thread.is_alive():
        time.sleep(0.05)
        new_ratio = ratio_delegate()
        if new_ratio != ratio:
            print_progress(ratio)
            ratio = new_ratio
    thread.join()
    print(PREV_LINE, end='')


class ConsoleStyles:
    BOLD = '\033[1m'
    GRAY = '\033[90m'
    ERROR = '\033[91m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    OKBLUE = '\033[94m'
    HEADER = '\033[95m'
    ENDC = '\033[0m'


def main():
    any_alert = False
    version = sys.version_info
    msg = ''
    if (version.major, version.minor) < (3, 11):
        any_alert = True
        version = sys.version_info
        version = str(version.major) + '.' + str(version.minor)
        version += ' ' * (12 - len(version))
        msg += """
    | You are using Python {version} |
    | Python 3.11 is recommended        |
    | Some features may be unavailable  |
    +-----------------------------------+""".format(version=version)

    shortcuts_enabled = True
    try:
        win32com_client = imp.import_module('win32com.client')
    except ImportError:
        any_alert = True
        shortcuts_enabled = False
        msg += """
    | Module win32com not found         |
    | Shortcut feature is unavailable   |
    | Run "pip install pywin32" to fix  |
    +-----------------------------------+"""

    if any_alert:
        print(
            """
    +---------------ALERT---------------+"""
            + msg
        )
        input('\nPress Enter to continue...')
        print('\n')

    print(
        """{h}Welcome to NoitaSaves!{e}\n
> {w}To make a save, you should first quit the game{e}
> {w}You also need to close Noita before loading a save{e}
> {w}Turn off Steam sync in the game settings{e} (if it's enabled)
  > {w}Otherwise, do not load a save during Steam sync, it may corrupt the current game state{e}
  > {w}If the selected save has not loaded, just load it one more time{e}
    (It may happen due to steam sync)
> {w}You can also create a shortcut for NoitaSaves on your start menu or desktop{e}
  (Check github page for more info: {b}https://github.com/Sedo-KFM/NoitaSaves{e})""".format(
            h=ConsoleStyles.HEADER, w=ConsoleStyles.WARNING, e=ConsoleStyles.ENDC, b=ConsoleStyles.BOLD
        )
    )

    if not os.path.exists(SAVES_DIR):
        os.mkdir(SAVES_DIR)

    saves = []
    error_message = ''
    command = ''
    first_time = True
    while command != 'q':
        if error_message != '':
            input('\n' + ConsoleStyles.ERROR + 'Error: ' + error_message + ConsoleStyles.ENDC + '\n\nPress Enter...')
            error_message = ''

        if CLEAR_SCREEN:
            if first_time:
                first_time = False
            else:
                os.system('cls')

        print('\n\nSaves:')
        print('Loading...')
        saves = [(save, os.path.getctime(SAVES_DIR + '\\' + save)) for save in os.listdir(SAVES_DIR)]
        saves.sort(key=lambda s: s[1])
        current_save_size = calc_folder_size(CURRENT_SAVE)
        print(PREV_LINE, end='')
        for index, (save, creation_time) in enumerate(saves):
            is_equal_to_current = False
            save_size = get_folder_size_from_info_cargo or_recalc_new(SAVES_DIR + '\\' + save)
            if save_size == current_save_size:
                dir_cmp_result = dircmp(CURRENT_SAVE, SAVES_DIR + '\\' + save)
                if len(dir_cmp_result.diff_files) == 0:
                    is_equal_to_current = True
            printing_save = save.replace('_', ' ')
            print(
                ConsoleStyles.BOLD + ConsoleStyles.OKGREEN if is_equal_to_current else '',
                '#',
                index + 1,
                ' ' if index < 9 else '',
                ' >> ',
                printing_save,
                '' if is_equal_to_current else ConsoleStyles.ENDC + ConsoleStyles.GRAY,
                '  [',
                ' '.join(time.ctime(creation_time).replace('  ', ' ').split(' ')[1:4]),
                ' | ',
                '{:1.2f}'.format(save_size / 1000 / 1000),
                ' Mb',
                ']',
                ConsoleStyles.ENDC,
                sep='',
            )
        if len(saves) == 0:
            print('<< Nothing >>')
        print()

        command = ''
        parameter = ''
        user_input = input('S (Save) | L (Load) | P (Play) | D (Delete) | Q (Quit) >> ')
        user_input = user_input.lower().strip().split(maxsplit=1)
        print()
        if not user_input:
            continue
        if len(user_input) > 0:
            command = user_input[0]
        if len(user_input) > 1:
            parameter = user_input[1]
        if parameter == '' and command[0] in ('l', 'd'):
            parameter = command[1:]
            command = command[0]

        if command in ('l', 's', 'd'):  # :D
            if not os.path.exists(GAME_DIR):
                error_message = 'Game files not found'
                continue

            if command == 's':
                if not os.path.exists(CURRENT_SAVE):
                    error_message = 'Current progress not found\nTry to load any save or start a new game'
                    continue
                if parameter == '':
                    save_name = input('Input save name >> ')
                else:
                    save_name = parameter
                save_name_errors = set(re.findall(r'[^A-Za-zА-Яа-я0-9\- ]', save_name))
                if len(save_name_errors) > 0:
                    error_message = 'Incorrect symbols: [' + '], ['.join(save_name_errors) + ']'
                    continue
                save_name = save_name.replace(' ', '_')
                dirname = SAVES_DIR + '\\' + save_name
                if save_name in os.listdir(SAVES_DIR):
                    if input('Overwrite? [y/N] >> ').strip().lower() != 'y':
                        continue
                    shutil.rmtree(dirname)
                print('Saving...')
                target_containment = calc_folder_containment(CURRENT_SAVE)
                run_with_progress(
                    lambda: shutil.copytree(CURRENT_SAVE, dirname),
                    lambda: calc_folder_containment(dirname) / target_containment,
                )
                with open(dirname + '\\' + INFO_FILE_NAME, 'w', encoding='utf-8') as info_file:
                    info_file.write(str(calc_folder_size(dirname)))

            elif command in ('l', 'd'):
                if len(saves) == 0:
                    error_message = 'Nothing to ' + ('load' if command == 'l' else 'delete')
                    continue
                save_index = None
                if parameter == '':
                    parameter = input(
                        'Select the save index ' + ('(or "a" (all))' if command == 'd' else '(or "l" (last))') + ' >> '
                    )
                if parameter.isdecimal():
                    save_index = int(parameter)
                    if not 0 < save_index <= len(saves):
                        error_message = 'Incorrect index'
                        continue
                elif command == 'd' and parameter == 'a':
                    print(f'{ConsoleStyles.WARNING}Are you sure?{ConsoleStyles.ENDC} [y/N]', end=' >> ')
                    if input().strip().lower() != 'y':
                        continue
                elif command == 'd' and '-' in parameter:
                    start, end = parameter.split('-')
                    if not start.isdecimal() or not end.isdecimal() or not (0 < int(start) <= int(end) <= len(saves)):
                        error_message = 'Incorrect index'
                        continue
                    save_index = (int(start), int(end))
                elif command == 'l' and parameter == 'l':
                    save_index = len(saves)
                else:
                    error_message = 'Incorrect index'
                    continue

                if command == 'l':
                    print('Loading...')
                    if os.path.exists(CURRENT_SAVE):
                        shutil.rmtree(CURRENT_SAVE)
                    selected_save = SAVES_DIR + '\\' + saves[save_index - 1][0]
                    target_containment = calc_folder_containment(selected_save)
                    run_with_progress(
                        lambda: shutil.copytree(selected_save, CURRENT_SAVE),
                        lambda: calc_folder_containment(CURRENT_SAVE) / target_containment,
                    )
                    if INFO_FILE_NAME in os.listdir(CURRENT_SAVE):
                        os.remove(CURRENT_SAVE + '\\' + INFO_FILE_NAME)

                elif command == 'd':
                    print('Deleting...')
                    if parameter == 'a':
                        rm_saves = [save for save, _ in saves]
                    elif '-' in parameter:
                        rm_saves = [save for save, _ in saves[save_index[0] - 1 : save_index[1]]]
                    else:
                        rm_saves = [saves[save_index - 1][0]]
                    initial_content = sum(calc_folder_containment(SAVES_DIR + '\\' + save) for save in rm_saves)
                    run_with_progress(
                        lambda: [shutil.rmtree(SAVES_DIR + '\\' + save) for save in rm_saves],
                        lambda: 1
                        - (
                            sum(calc_folder_containment(SAVES_DIR + '\\' + save) for save in rm_saves) / initial_content
                        ),
                    )
            print('Done!')

        elif command in ('sda', 'sma', 'sdr', 'smr'):
            if not shortcuts_enabled:
                error_message = 'Shortcut feature is disabled'
                continue

            if 'd' in command:
                shortcut_path = os.getenv('USERPROFILE') + r'\Desktop'
            elif 'm' in command:
                shortcut_path = os.getenv('APPDATA') + r'\Microsoft\Windows\Start Menu\Programs'
            shortcut_path += '\\' + FILENAME + '.lnk'
            shortcut_removed = False
            if os.path.exists(shortcut_path):
                os.remove(shortcut_path)
                shortcut_removed = True

            if 'a' in command:
                shell = win32com_client.Dispatch('WScript.Shell')
                shortcut = shell.CreateShortCut(shortcut_path)
                shortcut.Targetpath = os.getcwd() + '\\' + FILENAME + '.py'
                shortcut.IconLocation = os.getcwd() + '\\' + FILENAME + '.ico'
                shortcut.WorkingDirectory = os.getcwd()
                shortcut.save()
                print('\nShortcut ' + ('updated' if shortcut_removed else 'created') + '!')
            else:
                print('\nShortcut removed!')

        elif command == 'p':
            os.system('start steam://rungameid/881100')

        else:
            if len(command) > 0:
                error_message = 'Incorrect command'

    if len(saves) == 0:
        os.rmdir(SAVES_DIR)


if __name__ == '__main__':
    try:
        main()
    except Exception as e:
        print(ConsoleStyles.ERROR + 'CRITICAL ERROR:' + ConsoleStyles.ENDC)
        print(e)
        input('Press Enter to exit...')
