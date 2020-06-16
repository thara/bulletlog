import os, subprocess, textwrap, shutil, traceback
import hashlib

sut = 'target/debug/bulletlog'
root_dir = 'tmp'

def mkdir(path):
    try:
        os.mkdir(path)
    except FileExistsError:
        pass
    return path

def ensure(func, path):
    try:
        func(path)
    except (FileExistsError, FileNotFoundError):
        pass
    return path

def bulletlog_env(file, date):
    return {'BULLETLOG_FILE': file, 'BULLETLOG_DATE': date}

def run(arg, file=None, date=None, **kwargs):
    env = {'BULLETLOG_FILE': file, 'BULLETLOG_DATE': date}
    env = {k: v for (k, v) in env.items() if v is not None}
    return subprocess.run(sut + " " +arg, shell=True, check=True, env=env, **kwargs)

def sha1(b):
    return hashlib.sha1(b).hexdigest()

def test_new_file():
    dir = mkdir(os.path.join(root_dir, 'test_new_file'))
    target_file = ensure(os.remove, os.path.join(dir, 'result'))

    run("add NOTE", target_file, '2020-01-05')

    expected = textwrap.dedent("""\
    ## 2020-01-05

    * NOTE

    """)
    with open(target_file, 'rb') as f:
        assert sha1(expected.encode('utf-8')) == sha1(f.read())

def test_add_entry():
    dir = mkdir(os.path.join(root_dir, 'test_add_entry'))
    target_file = ensure(os.remove, os.path.join(dir, 'result'))

    run("task NOTE1", target_file, '2020-01-05')
    run("task NOTE2", target_file, '2020-01-05')
    run("task NOTE3", target_file, '2020-01-05')

    expected = textwrap.dedent("""\
    ## 2020-01-05

    - NOTE1

    - NOTE2

    - NOTE3

    """)
    with open(target_file, 'rb') as f:
        assert sha1(expected.encode('utf-8')) == sha1(f.read())

def test_new_section():
    dir = mkdir(os.path.join(root_dir, 'test_new_section'))
    target_file = ensure(os.remove, os.path.join(dir, 'result'))

    run("task NOTE1", target_file, '2020-01-05')
    run("task NOTE2", target_file, '2020-01-05')
    run("task NOTE3", target_file, '2020-01-05')
    run("add NOTE5", target_file, '2020-01-10')

    expected = textwrap.dedent("""\
    ## 2020-01-10

    * NOTE5

    ## 2020-01-05

    - NOTE1

    - NOTE2

    - NOTE3

    """)
    with open(target_file, 'rb') as f:
        assert sha1(expected.encode('utf-8')) == sha1(f.read())

def test_tasks():
    dir = mkdir(os.path.join(root_dir, 'test_tasks'))
    target_file = ensure(os.remove, os.path.join(dir, 'result'))

    run("task NOTE1", target_file, '2020-01-05')
    run("add NOTE2", target_file, '2020-01-05')
    run("task NOTE3", target_file, '2020-01-05')
    run("add NOTE5", target_file, '2020-01-10')
    run("task NOTE8", target_file, '2020-01-10')
    run("add NOTE10", target_file, '2020-01-25')
    run("task NOTE12", target_file, '2020-01-25')

    r = run("tasks", target_file, capture_output=True)

    expected = textwrap.dedent("""\
    0: NOTE12
    1: NOTE8
    2: NOTE1
    3: NOTE3
    """)
    assert sha1(expected.encode('utf-8')) == sha1(r.stdout)

def test_complete_task():
    dir = mkdir(os.path.join(root_dir, 'test_complete_task'))
    target_file = ensure(os.remove, os.path.join(dir, 'result'))

    run("task NOTE1", target_file, '2020-01-05')
    run("add NOTE2", target_file, '2020-01-05')
    run("add NOTE5", target_file, '2020-01-10')
    run("task NOTE8", target_file, '2020-01-10')
    run("add NOTE10", target_file, '2020-01-25')
    run("task NOTE12", target_file, '2020-01-25')

    run("comp 2", target_file)

    expected = textwrap.dedent("""\
    ## 2020-01-25

    * NOTE10

    - NOTE12

    ## 2020-01-10

    * NOTE5

    - NOTE8

    ## 2020-01-05

    x NOTE1

    * NOTE2

    """)

    with open(target_file, 'rb') as f:
        assert sha1(expected.encode('utf-8')) == sha1(f.read())

# Before all
shutil.rmtree(root_dir)
mkdir(root_dir)

test_failed = False

try:
    test_new_file()
    test_add_entry()
    test_new_section()
    test_tasks()
    test_complete_task()
except AssertionError as e:
    test_failed = True
    traceback.print_exec()

if test_failed:
    print("failed")
else:
    print("pass")
