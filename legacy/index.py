import time
from datetime import datetime

import psutil
import win32gui
import win32process
import win32api
import tkinter
from tkinter import messagebox, BitmapImage, Label, Button
import threading
import re
import json
from tkinter import *
from tkinter.ttk import *
from tkinter import ttk
import ctypes
import requests as rq
server = "serverp.furtorch.heili.tech"
import os
if os.path.exists("config.json") == False:
    with open("config.json", "w", encoding="utf-8") as f:
        config_data = {
            "cost_per_map": 0,
            "opacity": 1.0,
            "tax": 0,
        }
        json.dump(config_data, f, ensure_ascii=False, indent=4)

config_data = {}
def convert_from_log_structure(log_text: str, verbose: bool = False):
    """
    将结构化日志文本转换为嵌套字典

    参数:
        log_text: 包含结构化日志的文本
        verbose: 是否输出详细日志信息

    返回:
        转换后的嵌套字典
    """
    # 分割并过滤空行
    lines = [line.strip() for line in log_text.split('\n') if line.strip()]
    stack  = []
    root = {}

    if verbose:
        print("=== 开始解析 ===")

    for line in lines:
        # 计算层级（'|'的数量）
        level = line.count('|')
        # 提取内容（移除所有'|'并修剪）
        content = re.sub(r'\|+', '', line).strip()

        if verbose:
            print(f"\n处理: '{line}'")
            print(f"  层级: {level}, 内容: '{content}'")

        # 调整栈以匹配当前层级
        while len(stack) > level:
            stack.pop()

        # 确定父节点
        if not stack:
            parent = root
        else:
            parent = stack[-1]

        # 跳过空父节点
        if parent is None:
            continue

        # 解析键值对（包含[]的情况）
        if '[' in content and ']' in content:
            # 提取键部分和值部分
            key_part = content[:content.index('[')].strip()
            value_part = content[content.index('[') + 1: content.rindex(']')].strip()

            # 转换值类型
            if value_part.lower() == 'true':
                value = True
            elif value_part.lower() == 'false':
                value = False
            elif re.match(r'^-?\d+$', value_part):
                value = int(value_part)
            else:
                value = value_part

            # 处理多级键（用'+'分隔）
            keys = [k.strip() for k in key_part.split('+') if k.strip()]

            current_node = parent

            for i in range(len(keys)):
                key = keys[i]
                # 跳过空键
                if not key:
                    continue

                # 检查当前节点是否有效
                if current_node is None:
                    continue

                if i == len(keys) - 1:
                    # 最后一个键，设置值
                    current_node[key] = value
                else:
                    # 不是最后一个键，确保是字典并创建子节点
                    if not isinstance(current_node, dict):
                        break

                    if key not in current_node:
                        current_node[key] = {}
                    current_node = current_node[key]

                    # 检查新节点是否有效
                    if current_node is None:
                        break

            # 将当前节点加入栈
            stack.append(current_node)

        # 处理没有值的键（如 +SpecialInfo）
        else:
            key_part = content.strip()
            keys = [k.strip() for k in key_part.split('+') if k.strip()]

            current_node = parent

            for key in keys:
                # 跳过空键
                if not key:
                    continue

                # 检查当前节点是否有效
                if current_node is None:
                    continue

                # 确保当前节点是字典
                if not isinstance(current_node, dict):
                    break

                # 创建子节点（如果不存在）
                if key not in current_node:
                    current_node[key] = {}
                current_node = current_node[key]

                # 检查新节点是否有效
                if current_node is None:
                    break

            # 将当前节点加入栈
            stack.append(current_node)

    if verbose:
        print("\n=== 解析完成 ===")

    return root


def log_to_json(log_text):
    """将日志文本转换为JSON字符串"""
    parsed_data = convert_from_log_structure(log_text)
    #return json.dumps(parsed_data, indent=4, ensure_ascii=False)
    return parsed_data

def get_price_info(text):
    try:
        pattern_id = r'XchgSearchPrice----SynId = (\d+).*?\+refer \[(\d+)\]'
        match = re.findall(pattern_id, text, re.DOTALL)
        result = list(match)
        for i, item in enumerate(result, 1):
            ids = item[1]
            synid = item[0]
            pattern = re.compile(
                rf'----Socket RecvMessage STT----XchgSearchPrice----SynId = {synid}\s+'  # 匹配目标SynId
                r'\[.*?\]\s*GameLog: Display: \[Game\]\s+'  # 匹配时间和固定前缀
                r'(.*?)(?=----Socket RecvMessage STT----|$)',  # 匹配数据块内容（到下一个数据块或结束）
                re.DOTALL  # 允许.匹配换行
            )

            # 查找目标数据块
            match = pattern.search(text)
            data_block = match.group(1)
            if not match:
                print(f'发现记录： ID:{item[1]}, 价格:-1')
            if int(item[1]) == 100300:
                continue
            # 提取所有+数字 [数值]中的数值（忽略currency）
            value_pattern = re.compile(r'\+\d+\s+\[([\d.]+)\]')  # 匹配+数字 [x.x]格式
            values = value_pattern.findall(data_block)
            # 获得前30个values的平均值，但若values的长度小于30，则取全部的平均值
            if len(values) == 0:
                average_value = -1
            else:
                num_values = min(len(values), 30)
                sum_values = sum(float(values[i]) for i in range(num_values))
                average_value = sum_values / num_values
            with open("full_table.json", 'r', encoding="utf-8") as f:
                full_table = json.load(f)
                try:
                    full_table[ids]['last_time'] = round(time.time())
                    #full_table[ids]['from'] = "Local"
                    full_table[ids]['from'] = "FurryHeiLi"
                    full_table[ids]['price'] = round(average_value, 4)
                except:
                    pass
            with open("full_table.json", 'w', encoding="utf-8") as f:
                json.dump(full_table, f, indent=4, ensure_ascii=False)
            print(f'更新物品价值： ID:{ids}, 名称:{full_table[ids]["name"]}, 价格:{round(average_value, 4)}')
            price_submit(ids, round(average_value, 4), get_user())
    except Exception as e:
        print(e)




all_time_passed = 1

hwnd = win32gui.FindWindow(None, "Torchlight: Infinite  ")
tid, pid = win32process.GetWindowThreadProcessId(hwnd)
process = psutil.Process(pid)
position_game = process.exe()
position_log = position_game + "/../../../TorchLight/Saved/Logs/UE_game.log"
position_log = position_log.replace("\\", "/")
print(position_log)
with open(position_log, "r", encoding="utf-8") as f:
    print(f.read(100))
    # 翻到文件末尾
    f.seek(0, 2)
exclude_list = []

def scanned_log(changed_text):
    lines = changed_text.split('\n')
    drop_blocks = []
    i = 0
    line_count = len(lines)

    while i < line_count:
        line = lines[i]
        # 匹配起始标记：+DropItems+1+（使用大小写敏感匹配）
        if re.search(r'\+DropItems\+1\+', line):
            # 初始化当前块，包含起始行
            current_block = [line]
            j = i + 1

            # 收集后续行直到遇到结束标记
            while j < line_count:
                current_line = lines[j]

                # 遇到包含"Display:"的行时，结束当前块（包含此行）
                if 'Display:' in current_line:
                    current_block.append(current_line)
                    j += 1
                    break

                # 收集所有相关行（包括子行和同级行）
                current_block.append(current_line)
                j += 1

            # 将当前块的所有行用换行符连接后添加到结果列表
            drop_blocks.append('\n'.join(current_block))
            # 移动索引到当前块结束的下一行
            i = j
        else:
            # 未找到起始标记，继续检查下一行
            i += 1
    return drop_blocks

pending_items = {}
def deal_drop(drop_data, item_id_table, price_table):
    """更新掉落统计信息"""
    global income, income_all, drop_list, drop_list_all
    def invoke_drop_item_processing(item_data, item_key):
        global income, income_all, drop_list, drop_list_all, exclude_list, pending_items, config_data
        """处理单个掉落物品数据"""
        # 检查是否被拾取（Picked可能在根级别或item内部）
        picked = False
        print(item_data)
        if "Picked" in item_data:
            picked = item_data["Picked"]
        elif isinstance(item_data.get("item"), dict) and "Picked" in item_data["item"]:
            picked = item_data["item"]["Picked"]

        if not picked:
            return

        # 处理SpecialInfo（嵌套物品信息）
        item_info = item_data.get("item", {})
        if isinstance(item_info, dict) and "SpecialInfo" in item_info:
            special_info = item_info["SpecialInfo"]
            if isinstance(special_info, dict):
                if "BaseId" in special_info:
                    item_info["BaseId"] = special_info["BaseId"]
                if "Num" in special_info:
                    item_info["Num"] = special_info["Num"]

        # 获取基础ID和数量
        base_id = item_info.get("BaseId")
        num = item_info.get("Num", 0)

        if base_id is None:
            return

        # 转换ID为名称
        base_id_str = str(base_id)
        item_name = base_id_str  # 默认用ID作为名称

        if base_id_str in item_id_table:
            item_name = item_id_table[base_id_str]
        else:
            # 本地无数据，加入待处理队列
            global pending_items
            if base_id_str not in pending_items:
                print(f"[网络] ID {base_id_str} 本地不存在，启动获取")
                pending_items[base_id_str] = num
            else:
                pending_items[base_id_str] += num
                print(f"[网络] ID {base_id_str} 已在队列，累计: {pending_items[base_id_str]}")
            return

        # 检查物品名称是否为空
        if not item_name.strip():
            return

        # 检查是否在排除列表
        global exclude_list
        if exclude_list and item_name in exclude_list:
            print(f"已排除: {item_name} x{num}")
            return
        print(base_id)
        # 统计数量
        if base_id not in drop_list:
            drop_list[base_id] = 0
        drop_list[base_id] += num

        if base_id not in drop_list_all:
            drop_list_all[base_id] = 0
        drop_list_all[base_id] += num

        # 计算价格
        price = 0.0
        if str(base_id) in price_table:
            base_id = str(base_id)
            price = price_table[base_id]
            if config_data.get("tax", 0) == 1 and base_id != "100300":
                price = price * 0.875
            income += price * num
            income_all += price * num

        # 记录到文件
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        log_line = f"[{timestamp}] 掉落: {item_name} x{num} 份 ({round(price, 3)}/份)\n"
        with open("drop.txt", "a", encoding="utf-8") as f:
            f.write(log_line)

    def invoke_drop_items_recursive(data, path=""):
        """递归处理所有掉落项"""
        for key, value in data.items():
            current_path = f"{path}.{key}" if path else key

            # 检查是否包含掉落数据
            if isinstance(value, dict) and "item" in value:
                # 判断是否有Picked标记
                has_picked = ("Picked" in value) or \
                             (isinstance(value["item"], dict) and "Picked" in value["item"])

                if has_picked:
                    invoke_drop_item_processing(value, current_path)

            # 递归处理子项
            if isinstance(value, dict):
                invoke_drop_items_recursive(value, current_path)

    # 启动递归处理
    invoke_drop_items_recursive(drop_data)
def deal_change(changed_text):
    global root
    global is_in_map, all_time_passed, drop_list, income, t, drop_list_all, income_all, total_time, map_count
    if "PageApplyBase@ _UpdateGameEnd: LastSceneName = World'/Game/Art/Maps/01SD/XZ_YuJinZhiXiBiNanSuo200/XZ_YuJinZhiXiBiNanSuo200.XZ_YuJinZhiXiBiNanSuo200' NextSceneName = World'/Game/Art/Maps" in changed_text:
        is_in_map = True
        drop_list = {}
        income = -root.cost
        income_all += -root.cost
        map_count += 1
    if "NextSceneName = World'/Game/Art/Maps/01SD/XZ_YuJinZhiXiBiNanSuo200/XZ_YuJinZhiXiBiNanSuo200.XZ_YuJinZhiXiBiNanSuo200'" in changed_text:
        is_in_map = False
        total_time += time.time() - t
    texts = changed_text
    id_table = {}
    price_table = {}
    with open("full_table.json", 'r', encoding="utf-8") as f:
        f = json.load(f)
    for i in f.keys():
        id_table[str(i)] = f[i]["name"]
        price_table[str(i)] = f[i]["price"]
    texts = scanned_log(texts)
    if texts == []:
        return
    for text in texts:
        text = convert_from_log_structure(text)
        deal_drop(text, id_table, price_table)
    print(texts)
    if texts != []:
        root.reshow()
        if is_in_map == False:
            is_in_map = True

is_in_map = False
drop_list = {}
drop_list_all = {}
income = 0
income_all = 0
t = time.time()
show_all = False
total_time = 0
map_count = 0

class App(Tk):
    show_type = ["罗盘","硬通货","特殊道具","追忆材料","装备材料","玩法门票","地图门票","魔方材料","侵蚀材料","做梦材料","高塔材料","BOSS 门票","记忆荧光","神威纹章","叠界材料"]
    # 对，圈，错
    status = ["✔", "◯", "✘"]
    cost = 0
    def __init__(self):
        super().__init__()
        self.title("FurTorch v0.0.1a4")
        self.geometry()

        ctypes.windll.shcore.SetProcessDpiAwareness(1)
        # 调用api获得当前的缩放因子
        ScaleFactor = ctypes.windll.shcore.GetScaleFactorForDevice(0)
        # 设置缩放因子
        self.tk.call('tk', 'scaling', ScaleFactor / 75)
        basic_frame = ttk.Frame(self)
        advanced_frame = ttk.Frame(self)
        basic_frame.pack(side="top", fill="both")
        advanced_frame.pack(side="top", fill="both")
        self.basic_frame = basic_frame
        self.advanced_frame = advanced_frame
        # 去掉窗口最大化按钮
        self.resizable(False, False)
        # 去掉窗口最小化按钮
        self.attributes('-toolwindow', True)
        # 设置红色
        basic_frame.config(style="Red.TFrame")
        advanced_frame.config(style="Blue.TFrame")
        style = ttk.Style()
        #style.configure("Red.TFrame", background="#ffcccc")
        #style.configure("Blue.TFrame", background="#ccccff")
        label_current_time = ttk.Label(basic_frame, text="当前：0m00s", font=("黑体", 14), anchor="w")
        label_current_time.grid(row=0, column=0, padx = 5, sticky="w")
        label_current_speed = ttk.Label(basic_frame, text="🔥 0 /分", font=("黑体", 14))
        label_current_speed.grid(row=0, column=2, sticky="e", padx = 5)
        label_total_time = ttk.Label(basic_frame, text="总计：00m00s", font=("黑体", 14), anchor="w")
        label_total_time.grid(row=1, column=0, padx = 5, sticky="w")
        label_total_speed = ttk.Label(basic_frame, text="🔥 0 /分", font=("黑体", 14))
        label_total_speed.grid(row=1, column=2, sticky="e", padx = 5)
        self.label_current_time = label_current_time
        self.label_current_speed = label_current_speed
        self.label_total_time = label_total_time
        self.label_total_speed = label_total_speed
        # 一条线
        separator = ttk.Separator(basic_frame, orient='horizontal')
        separator.grid(row=2, columnspan=3, sticky="ew", pady=5)
        # 标签 占据两格
        label_current_earn = ttk.Label(basic_frame, text="🔥 0", font=("Algerian", 20, "bold"))
        label_current_earn.grid(row=3, column=0, padx=5)
        label_map_count = ttk.Label(basic_frame, text="🎫 0", font=("黑体", 14))
        label_map_count.grid(row=3, column=1, padx=5)
        # 按钮 占据一格
        words_short = StringVar()
        words_short.set("当前地图")
        self.words_short = words_short
        button_show_advanced = ttk.Button(basic_frame, textvariable=words_short)
        button_show_advanced.grid(row=3, column=2, padx=5)
        button_show_advanced.config(command=self.change_states)
        self.label_current_earn = label_current_earn
        self.label_map_count = label_map_count
        self.button_show_advanced = button_show_advanced

        # 按钮 掉落 过滤 日志 设置 高度和宽度相等
        button_drops = ttk.Button(advanced_frame, text="掉落", width=7)
        button_filter = ttk.Button(advanced_frame, text="过滤", width=7)
        button_log = ttk.Button(advanced_frame, text="日志", width=7)
        button_settings = ttk.Button(advanced_frame, text="设置", width=7)
        button_drops.grid(row=0, column=0, padx=5, ipady=10)
        button_filter.grid(row=0, column=1, padx=5, ipady=10)
        button_log.grid(row=0, column=2, padx=5, ipady=10)
        button_settings.grid(row=0, column=3, padx=5, ipady=10)
        # 新窗口四个
        self.button_drops = button_drops
        self.button_filter = button_filter
        self.button_log = button_log
        self.button_settings = button_settings

        self.button_settings.config(command=self.show_settings, cursor="hand2")
        self.button_drops.config(command=self.show_diaoluo, cursor="hand2")

        self.inner_pannel_drop = Toplevel(self)
        self.inner_pannel_drop.title("掉落")
        self.inner_pannel_drop.geometry()
        # 隐藏最大化和最小化按钮
        self.inner_pannel_drop.resizable(False, False)
        self.inner_pannel_drop.attributes('-toolwindow', True)
        # 移动至主窗口右侧
        self.inner_pannel_drop.geometry('+0+0')
        inner_pannel_drop_left = ttk.Frame(self.inner_pannel_drop)
        inner_pannel_drop_left.grid(row=0, column=0)
        words = StringVar()
        words.set("目前：当前地图掉落 点击切换总掉落")
        inner_pannel_drop_show_all = ttk.Button(self.inner_pannel_drop, textvariable=words, width=30)
        inner_pannel_drop_show_all.grid(row=0, column=1)
        self.words = words
        self.inner_pannel_drop_show_all = inner_pannel_drop_show_all
        self.inner_pannel_drop_show_all.config(cursor="hand2", command=self.change_states)
        inner_pannel_drop_right = ttk.Frame(self.inner_pannel_drop)
        inner_pannel_drop_right.grid(row=1, column=1, rowspan=5)
        inner_pannel_drop_total = ttk.Button(self.inner_pannel_drop, text="全部", width=7)
        inner_pannel_drop_total.grid(row=0, column=0, padx=5, ipady=10)
        inner_pannel_drop_tonghuo = ttk.Button(self.inner_pannel_drop, text="通货", width=7)
        inner_pannel_drop_tonghuo.grid(row=1, column=0, padx=5, ipady=10)
        inner_pannel_drop_huijing = ttk.Button(self.inner_pannel_drop, text="灰烬", width=7)
        inner_pannel_drop_huijing.grid(row=2, column=0, padx=5, ipady=10)
        inner_pannel_drop_luopan = ttk.Button(self.inner_pannel_drop, text="罗盘", width=7)
        inner_pannel_drop_luopan.grid(row=3, column=0, padx=5, ipady=10)
        inner_pannel_drop_yingguang = ttk.Button(self.inner_pannel_drop, text="荧光", width=7)
        inner_pannel_drop_yingguang.grid(row=4, column=0, padx=5, ipady=10)
        inner_pannel_drop_qita = ttk.Button(self.inner_pannel_drop, text="其他", width=7)
        inner_pannel_drop_qita.grid(row=5, column=0, padx=5, ipady=10)
        self.inner_pannel_drop_total = inner_pannel_drop_total
        self.inner_pannel_drop_tonghuo = inner_pannel_drop_tonghuo
        self.inner_pannel_drop_huijing = inner_pannel_drop_huijing
        self.inner_pannel_drop_luopan = inner_pannel_drop_luopan
        self.inner_pannel_drop_yingguang = inner_pannel_drop_yingguang
        self.inner_pannel_drop_qita = inner_pannel_drop_qita
        self.inner_pannel_drop_total.config(command=self.show_all_type, cursor="hand2")
        self.inner_pannel_drop_tonghuo.config(command=self.show_tonghuo, cursor="hand2")
        self.inner_pannel_drop_huijing.config(command=self.show_huijing, cursor="hand2")
        self.inner_pannel_drop_luopan.config(command=self.show_luopan, cursor="hand2")
        self.inner_pannel_drop_yingguang.config(command=self.show_yingguang, cursor="hand2")
        self.inner_pannel_drop_qita.config(command=self.show_qita, cursor="hand2")
        # 纵向滚动条
        self.inner_pannel_drop_scrollbar = Scrollbar(inner_pannel_drop_right)
        self.inner_pannel_drop_scrollbar.config(orient=VERTICAL)
        self.inner_pannel_drop_scrollbar.pack(side=RIGHT, fill=Y)
        self.inner_pannel_drop_listbox = Listbox(inner_pannel_drop_right, yscrollcommand=self.inner_pannel_drop_scrollbar.set, width=50, height=20)
        self.inner_pannel_drop_listbox.pack(side=LEFT, fill=BOTH)
        self.inner_pannel_drop_scrollbar.config(command=self.inner_pannel_drop_listbox.yview)
        self.inner_pannel_drop_listbox.insert(END, f"{self.status[0]} <3min {self.status[1]} <15min {self.status[2]} >15min")
        # 设置行高
        self.inner_pannel_drop_listbox.config(font=("Consolas", 12))
        # 设置宽度
        self.inner_pannel_drop_listbox.config(width=30)

        # 设置页面
        self.inner_pannel_settings = Toplevel(self)
        self.inner_pannel_settings.title("设置")
        self.inner_pannel_settings.geometry()
        # 隐藏最大化和最小化按钮
        self.inner_pannel_settings.resizable(False, False)
        self.inner_pannel_settings.attributes('-toolwindow', True)
        # 移动至主窗口右侧
        self.inner_pannel_settings.geometry('+300+0')
        # Label + 文本框
        label_setting_1 = ttk.Label(self.inner_pannel_settings, text="单图成本:")
        label_setting_1.grid(row=0, column=0, padx=5, pady=5)
        entry_setting_1 = ttk.Entry(self.inner_pannel_settings)
        entry_setting_1.grid(row=0, column=1, padx=5, pady=5)
        global config_data
        # 选择计税 不计税
        with open("config.json", "r", encoding="utf-8") as f:
            config_data = f.read()
        config_data = json.loads(config_data)
        chose = ttk.Combobox(self.inner_pannel_settings, values=["不计税", "计税"], state="readonly")
        chose.current(config_data.get("tax", 0))
        chose.grid(row=2, column=1, padx=5, pady=5)
        self.chose = chose
        chose.bind("<<ComboboxSelected>>", lambda event: self.change_tax(self.chose.current()))
        self.label_setting_1 = label_setting_1
        self.entry_setting_1 = entry_setting_1
        # 设置透明度
        self.label_setting_2 = ttk.Label(self.inner_pannel_settings, text="透明度:")
        self.label_setting_2.grid(row=1, column=0, padx=5, pady=5)
        # 滑动条
        self.scale_setting_2 = ttk.Scale(self.inner_pannel_settings, from_=0.1, to=1.0, orient=HORIZONTAL)
        self.scale_setting_2.grid(row=1, column=1, padx=5, pady=5)
        self.scale_setting_2.config(command=self.change_opacity)
        print(config_data)
        self.entry_setting_1.insert(0, str(config_data["cost_per_map"]))
        self.entry_setting_1.bind("<Return>", lambda event: self.change_cost(self.entry_setting_1.get()))
        self.scale_setting_2.set(config_data["opacity"])
        self.change_opacity(config_data["opacity"])
        self.change_cost(config_data["cost_per_map"])
        self.inner_pannel_drop.withdraw()
        self.inner_pannel_settings.withdraw()
        self.inner_pannel_drop.protocol("WM_DELETE_WINDOW", self.close_diaoluo)
        self.inner_pannel_settings.protocol("WM_DELETE_WINDOW", self.close_settings)
        # 置顶
        self.attributes('-topmost', True)
        self.inner_pannel_drop.attributes('-topmost', True)
        self.inner_pannel_settings.attributes('-topmost', True)
    def change_tax(self, value):
        global config_data
        with open("config.json", "r", encoding="utf-8") as f:
            config_data = f.read()
        config_data = json.loads(config_data)
        config_data["tax"] = int(value)
        with open("config.json", "w", encoding="utf-8") as f:
            json.dump(config_data, f, ensure_ascii=False, indent=4)

    def change_states(self):
        global show_all
        show_all = not show_all
        if not show_all:
            self.words.set("目前：当前地图掉落 点击切换总掉落")
            self.words_short.set("当前地图")
        else:
            self.words.set("目前：总掉落 点击切换当前地图掉落")
            self.words_short.set("总掉落")
        self.reshow()
    def change_cost(self, value):
        value = str(value)
        with open("config.json", "r", encoding="utf-8") as f:
            config_data = f.read()
        config_data = json.loads(config_data)
        config_data["cost_per_map"] = float(value)
        with open("config.json", "w", encoding="utf-8") as f:
            print(config_data)
            json.dump(config_data, f, ensure_ascii=False, indent=4)
        self.cost = float(value)
    def show_diaoluo(self):
        this = self.inner_pannel_drop
        # 判断窗口是否隐藏
        if this.state() == "withdrawn":
            this.deiconify()
        else:
            this.withdraw()

    def close_diaoluo(self):
        self.inner_pannel_drop.withdraw()

    def close_settings(self):
        try:
            value = float(self.entry_setting_1.get())
            self.change_cost(value)
        except:
            pass
        self.inner_pannel_settings.withdraw()

    def show_settings(self):
        this = self.inner_pannel_settings
        if this.state() == "withdrawn":
            this.deiconify()
        else:
            this.withdraw()

    def change_opacity(self, value):
        with open("config.json", "r", encoding="utf-8") as f:
            config_data = f.read()
        config_data = json.loads(config_data)
        config_data["opacity"] = float(value)
        with open("config.json", "w", encoding="utf-8") as f:
            json.dump(config_data, f, ensure_ascii=False, indent=4)
        self.attributes('-alpha', float(value))
        self.inner_pannel_drop.attributes('-alpha', float(value))
        self.inner_pannel_settings.attributes('-alpha', float(value))
    def reshow(self):
        global drop_list, drop_list_all
        with open("full_table.json", 'r', encoding="utf-8") as f:
            full_table = json.load(f)
        self.label_map_count.config(text=f"🎫 {map_count}")
        if show_all:
            tmp = drop_list_all
            self.label_current_earn.config(text=f"🔥 {round(income_all, 2)}")
        else:
            tmp = drop_list
            self.label_current_earn.config(text=f"🔥 {round(income, 2)}")
        self.inner_pannel_drop_listbox.delete(1, END)
        for i in tmp.keys():

            item_id = str(i)
            item_name = full_table[item_id]["name"]
            item_type = full_table[item_id]["type"]
            if item_type not in self.show_type:
                continue
            now = time.time()
            last_time = full_table[item_id].get("last_update", 0)
            time_passed = now - last_time
            if time_passed < 180:
                status = self.status[0]
            elif time_passed < 900:
                status = self.status[1]
            else:
                status = self.status[2]
            item_price = full_table[item_id]["price"]
            if config_data.get("tax", 0) == 1 and item_id != "100300":
                item_price = item_price * 0.875
            self.inner_pannel_drop_listbox.insert(END, f"{status} {item_name} x{tmp[i]} [{tmp[i] * item_price}]")

    def show_all_type(self):
        self.show_type = ["罗盘","硬通货","特殊道具","追忆材料","装备材料","玩法门票","地图门票","魔方材料","侵蚀材料","做梦材料","高塔材料","BOSS 门票","记忆荧光","神威纹章","叠界材料"]
        self.reshow()
    def show_tonghuo(self):
        self.show_type = ["硬通货"]
        self.reshow()
    def show_huijing(self):
        self.show_type = ["装备材料"]
        self.reshow()
    def show_luopan(self):
        self.show_type = ["罗盘"]
        self.reshow()
    def show_yingguang(self):
        self.show_type = ["记忆荧光"]
        self.reshow()
    def show_qita(self):
        self.show_type = ["特殊道具","追忆材料","玩法门票","地图门票","魔方材料","侵蚀材料","做梦材料","高塔材料","BOSS 门票","神威纹章","叠界材料"]
        self.reshow()
class MyThread(threading.Thread):
    history = ""
    def run(self):
        global all_time_passed, income, drop_list, t, root
        self.history = open(position_log, "r", encoding="utf-8")
        self.history.seek(0, 2)
        while True:
            try:
                time.sleep(1)
                things = self.history.read()
                # print(things)
                deal_change(things)
                get_price_info(things)
                if is_in_map:
                    m = int((time.time() - t) // 60)
                    s = int((time.time() - t) % 60)
                    root.label_current_time.config(text=f"当前：{m}m{s}s")
                    root.label_current_speed.config(text=f"🔥 {round(income / ((time.time() - t) / 60), 2)} /分")
                    tmp_total_time = total_time + (time.time() - t)
                    m = int(tmp_total_time // 60)
                    s = int(tmp_total_time % 60)
                    root.label_total_time.config(text=f"总计：{m}m{s}s")
                    root.label_total_speed.config(text=f"🔥 {round(income_all / (tmp_total_time / 60), 2)} /分")
                else:
                    t = time.time()
            except Exception as e:
                print("-------------异常-----------")
                # 输出错误所在的行号
                import traceback
                traceback.print_exc()


def price_update():
    while True:
        try:
            r = rq.get(f"http://{server}/get", timeout=10).json()
            with open("full_table.json", 'w', encoding="utf-8") as f:
                json.dump(r, f, indent=4, ensure_ascii=False)
            print("价格更新成功")
            n = pending_items
            for i in n.keys():
                r = rq.get(f"http://{server}/gowork?id="+i, timeout=10).json()
                del pending_items[i]
                print(f"[网络] ID {i} 获取完成")
            time.sleep(90)
        except Exception as e:
            print("价格更新失败：" + str(e))
            time.sleep(10)



def price_submit(ids, price, user):
    print(price)
    try:
        r = rq.get(f"http://{server}/update?user={user}&ids={ids}&new_price={price}", timeout=10).json()
        print(r)
        return r
    except Exception as e:
        print(e)

def get_user():
    with open("config.json", "r", encoding="utf-8") as f:
        config_data = json.load(f)
    if not config_data.get("user", False):
        try:
            r = rq.get(f"http://{server}/reg", timeout=10).json()
            config_data["user"] = r["user_id"]
            user_id = r["user_id"]
            with open("config.json", "w", encoding="utf-8") as f:
                json.dump(config_data, f, ensure_ascii=False, indent=4)
        except:
            user_id = "3b95f1d6-5357-4efb-a96b-8cc3c76b3ee0"
    else:
        user_id = config_data["user"]
    return user_id


root = App()
root.wm_attributes('-topmost', 1)
MyThread().start()
import _thread
_thread.start_new_thread(price_update, ())
root.mainloop()