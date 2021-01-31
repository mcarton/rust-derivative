#!/usr/bin/env python

# This script is inspired from
# https://github.com/rust-lang/rust/blob/1f0fc02cc8ab4e0d9dd3e06a6d46fcb72b2a026f/src/etc/generate-deriving-span-tests.py

"""
This script creates a pile of UI tests check that all the
derives have spans that point to the fields, rather than the
#[derive(...)] line.
sample usage: src/etc/generate-deriving-span-tests.py
"""

import os
import stat

TEST_DIR = os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../../tests/compile-fail/generated/'))

try:
    os.mkdir(TEST_DIR)
except FileExistsError:
    pass

TEMPLATE = """\
// This file was auto-generated using 'src/etc/generate-deriving-span-tests.py'

#[cfg(feature = "use_core")]
extern crate core;

#[macro_use]
extern crate derivative;

{error_deriving}
struct Error;
{code}
fn main() {{}}
"""

ENUM_STRING = """
#[derive(Derivative)]
#[derivative({traits})]
enum Enum {{
   {default}
   A(Error)
}}
"""
ENUM_STRUCT_VARIANT_STRING = """
#[derive(Derivative)]
#[derivative({traits})]
enum Enum {{
   {default}
   A {{
     x: Error
   }}
}}
"""
STRUCT_STRING = """
#[derive(Derivative)]
#[derivative({traits})]
struct Struct {{
    x: Error
}}
"""
STRUCT_TUPLE_STRING = """
#[derive(Derivative)]
#[derivative({traits})]
struct Struct(
    Error
);
"""

ENUM_TUPLE, ENUM_STRUCT, STRUCT_FIELDS, STRUCT_TUPLE = range(4)


def create_test_case(type, trait, super_traits):
    string = [ENUM_STRING, ENUM_STRUCT_VARIANT_STRING, STRUCT_STRING, STRUCT_TUPLE_STRING][type]
    all_traits = ','.join([trait] + super_traits)
    super_traits = ','.join(super_traits)
    error_deriving = '#[derive(%s)]' % super_traits if super_traits else ''

    if trait == "Default" and (type == ENUM_TUPLE or type == ENUM_STRUCT):
        default = "#[derivative(Default)]"
    else:
        default = ""

    code = string.format(traits=all_traits, default=default)
    return TEMPLATE.format(error_deriving=error_deriving, code=code)


def write_file(name, string):
    test_file = os.path.join(TEST_DIR, 'derives-span-%s.rs' % name)

    # set write permission if file exists, so it can be changed
    if os.path.exists(test_file):
        os.chmod(test_file, stat.S_IWUSR)

    with open(test_file, 'w') as f:
        f.write(string)

    # mark file read-only
    os.chmod(test_file, stat.S_IRUSR|stat.S_IRGRP|stat.S_IROTH)


ENUM = 1
STRUCT = 2
ALL = STRUCT | ENUM

traits = {}

for (trait, supers) in [('Default', []),
                        ('Clone', []),
                        ('PartialEq', []),
                        ('PartialOrd', ['PartialEq']),
                        ('Eq, PartialEq', []), # Should be ('Eq', ['PartialEq]), see issue #85
                        ('Ord', ['Eq', 'PartialOrd', 'PartialEq']),
                        ('Debug', []),
                        ('Hash', [])]:
    traits[trait] = (ALL, supers)

for (trait, (types, super_traits)) in traits.items():
    mk = lambda ty: create_test_case(ty, trait, super_traits)
    if types & ENUM:
        write_file(trait + '-enum', mk(ENUM_TUPLE))
        write_file(trait + '-enum-struct-variant', mk(ENUM_STRUCT))
    if types & STRUCT:
        write_file(trait + '-struct', mk(STRUCT_FIELDS))
        write_file(trait + '-tuple-struct', mk(STRUCT_TUPLE))