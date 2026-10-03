import json
from google.protobuf import descriptor_pb2

data = open(r'C:\Users\Phitchayut\Desktop\ppsr_extracted\ppsr.dll', 'rb').read()
start = 0x1f07630
end = 0x1faea29
fd = descriptor_pb2.FileDescriptorProto()
fd.ParseFromString(data[start:end])

cmds = json.load(open('cmd_ids_4_6.json', encoding='utf-8'))

# Additional CmdIDs
EXTRA_CMDS = {
    'StartQuickCocoonStageRsp': 1371,
    'PlanetfesSendMsgCsReq': 3974,
}

TYPE_MAP = {
    1: 'double', 2: 'float', 3: 'int64', 4: 'uint64', 5: 'int32',
    6: 'fixed64', 7: 'fixed32', 8: 'bool', 9: 'string', 11: None,
    12: 'bytes', 13: 'uint32', 14: None,
    15: 'sfixed32', 16: 'sfixed64', 17: 'sint32', 18: 'sint64',
}

# Discover map entries
map_entries = {}
for m in fd.message_type:
    for n in m.nested_type:
        if n.options.map_entry:
            kf = next(f for f in n.field if f.number == 1)
            vf = next(f for f in n.field if f.number == 2)
            k_type = TYPE_MAP.get(kf.type) or kf.type_name.strip('.')
            v_type = TYPE_MAP.get(vf.type) or vf.type_name.strip('.')
            map_entries[f'{m.name}.{n.name}'] = (k_type, v_type)
            map_entries[n.name] = (k_type, v_type)

def format_type(f):
    if f.type_name:
        tname = f.type_name.strip('.')
        if tname in map_entries:
            k, v = map_entries[tname]
            return f'map<{k}, {v}>', True
        return tname, False
    return TYPE_MAP.get(f.type, 'bytes'), False

def emit_enum(enum, indent=''):
    lines = [f'{indent}enum {enum.name} {{']
    for v in enum.value:
        lines.append(f'{indent}    {v.name} = {v.number};')
    lines.append(f'{indent}}}')
    return lines

def emit_message(m, indent=''):
    lines = []
    cmd_name = 'Cmd' + m.name
    cmd_id = cmds.get(cmd_name) or EXTRA_CMDS.get(m.name)
    if cmd_id is not None:
        lines.append(f'{indent}// CmdID: {cmd_id}')
    lines.append(f'{indent}message {m.name} {{')
    
    for e in m.enum_type:
        lines.extend(emit_enum(e, indent + '    '))
    for n in m.nested_type:
        if not n.options.map_entry:
            lines.extend(emit_message(n, indent + '    '))
            
    # Custom message-specific field logic
    if m.name == 'SceneEntityInfo':
        # Emit non-entity fields first
        entity_fields = []
        for f in m.field:
            if f.name in ['Prop', 'Actor', 'SummonUnit', 'Npc', 'NpcMonster']:
                entity_fields.append(f)
            else:
                t_str, is_map = format_type(f)
                prefix = 'repeated ' if f.label == 3 else ''
                lines.append(f'{indent}    {prefix}{t_str} {f.name} = {f.number};')
        # Emit oneof entity
        lines.append(f'{indent}    oneof entity {{')
        name_map = {
            'Prop': 'prop',
            'Actor': 'actor',
            'SummonUnit': 'summon_unit',
            'Npc': 'npc',
            'NpcMonster': 'npc_monster',
        }
        for f in entity_fields:
            t_str, _ = format_type(f)
            fname = name_map.get(f.name, f.name.lower())
            lines.append(f'{indent}        {t_str} {fname} = {f.number};')
        lines.append(f'{indent}    }}')
        lines.append(f'{indent}}}')
        return lines

    if m.name == 'SceneEntityRefreshInfo':
        refresh_fields = []
        for f in m.field:
            if f.name in ['delete_entity', 'add_entity']:
                refresh_fields.append(f)
            else:
                t_str, is_map = format_type(f)
                prefix = 'repeated ' if f.label == 3 else ''
                lines.append(f'{indent}    {prefix}{t_str} {f.name} = {f.number};')
        lines.append(f'{indent}    oneof refresh_type {{')
        for f in refresh_fields:
            t_str, _ = format_type(f)
            lines.append(f'{indent}        {t_str} {f.name} = {f.number};')
        lines.append(f'{indent}    }}')
        lines.append(f'{indent}}}')
        return lines

    oneof_fields = {}
    for f in m.field:
        if f.HasField('oneof_index'):
            oneof_fields.setdefault(f.oneof_index, []).append(f)
        else:
            fname = f.name
            # Field name aliases for backwards compatibility with gameserver
            if m.name == 'PlayerSimpleInfo' and f.number == 12:
                fname = 'chat_bubble_id'
            elif m.name == 'GetFriendLoginInfoScRsp' and f.number == 6:
                fname = 'friend_uid_list'
            elif m.name == 'GetFriendLoginInfoScRsp' and f.number == 12:
                fname = 'black_uid_list'
            elif m.name == 'ReplaceLineupCsReq' and f.number == 15:
                fname = 'lineup_slot_list'
            
            t_str, is_map = format_type(f)
            if is_map:
                lines.append(f'{indent}    {t_str} {fname} = {f.number};')
            else:
                prefix = 'repeated ' if f.label == 3 else ''
                lines.append(f'{indent}    {prefix}{t_str} {fname} = {f.number};')
                
    for idx, o_fields in oneof_fields.items():
        if idx < len(m.oneof_decl):
            oname = m.oneof_decl[idx].name
            if m.name == 'SceneIdentifier' and oname == 'teleport_nice':
                oname = 'teleport_nigger'
            elif m.name == 'GetBigDataAllRecommendScRsp' and oname == 'OODKOEILJCD':
                oname = 'recommend_type'
                
            lines.append(f'{indent}    oneof {oname} {{')
            for f in o_fields:
                t_str, _ = format_type(f)
                lines.append(f'{indent}        {t_str} {f.name} = {f.number};')
            lines.append(f'{indent}    }}')
            
    lines.append(f'{indent}}}')
    return lines

all_lines = ['syntax = "proto3";\n']
for e in fd.enum_type:
    all_lines.extend(emit_enum(e))
    all_lines.append('')
for m in fd.message_type:
    all_lines.extend(emit_message(m))
    all_lines.append('')

output_file = 'proto/StarRail.proto'
with open(output_file, 'w', encoding='utf-8') as f:
    f.write('\n'.join(all_lines))

print(f'Wrote {len(all_lines)} lines to {output_file}')
