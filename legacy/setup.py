from distutils.core import setup
import py2exe
options = {
    'py2exe': {
        'includes': ['win32gui', 'win32process', 'tkinter', 'psutil', 're', 'json']
    }
}

setup(
    version="0.0.1a4",
    options=options,
    description="TorchFurry 火炬之光收益统计器 测试版",
    console=['index.py'],
    data_files=[('',['full_table.json',"更新日志.txt","注意事项.txt"])]
)