# SPDX-License-Identifier: AGPL-3.0-only
"""Metadata floor only; not a numerical comparator or exact-base ratchet."""
def validate(snapshot,current):
    errors=[]
    if len({s['unit']for s in current})!=len(current):return ['duplicate current unit']
    by_id={s['unit']:s for s in current}
    for old in snapshot:
        name=old['unit'];new=by_id.get(name)
        if new is None:errors.append(name+': missing parent suite');continue
        for key in ['test','directory']:
            if new[key]!=old[key]:errors.append(name+': changed parent '+key)
        old_ids=set(old['case_ids']);new_ids=set(new['case_ids'])
        if not old_ids<=new_ids:errors.append(name+': removed/replaced parent case')
        if not set(old['required_files'])<=set(new['required_files']):errors.append(name+': dropped raw file contract')
        degraded=set(new['refusals_without_rust_bin'])-set(old['refusals_without_rust_bin'])
        if degraded&old_ids:errors.append(name+': parent case lost Rust output obligation')
    return errors
