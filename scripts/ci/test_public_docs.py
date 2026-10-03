"""Regression checks for item identities in the public documentation gate."""

import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('public_docs', Path(__file__).with_name('check-public-docs.py'))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class PublicDocIdentityTests(unittest.TestCase):
    def key(self, source, declaration, kind='a struct field'):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'fixture.rs'
            path.write_text(source)
            start = source.index(declaration)
            message = {
                'message': 'missing documentation for ' + kind,
                'spans': [{'is_primary': True, 'expansion': None, 'file_name': str(path),
                           'byte_start': len(source[:start].encode()),
                           'byte_end': len(source[:start + len(declaration)].encode())}],
            }
            return checker.item_key(message).split('fixture.rs::', 1)[1]

    def test_line_moves_do_not_change_identity(self):
        source = 'pub struct Request { pub endpoint: String }'
        self.assertEqual(self.key(source, 'pub endpoint: String'),
                         self.key('// A comment with { braces } and unicode é\n' + source, 'pub endpoint: String'))

    def test_unrelated_documentation_cannot_offset_a_new_field(self):
        self.assertNotEqual(self.key('pub struct Request { pub endpoint: String }', 'pub endpoint: String'),
                            self.key('pub struct Request { pub timeout: u64 }', 'pub timeout: u64'))

    def test_fields_in_different_structs_have_different_identities(self):
        self.assertNotEqual(self.key('pub struct A { pub value: u64 }', 'pub value: u64'),
                            self.key('pub struct B { pub value: u64 }', 'pub value: u64'))

    def test_enum_variant_fields_have_distinct_namespaces(self):
        self.assertEqual(self.key('pub enum Error { Invalid { value: String } }', 'value: String'),
                         'enum Error/variant Invalid::a struct field::value')

    def test_tuple_variant_fields_have_distinct_namespaces(self):
        self.assertNotEqual(self.key('pub enum Error { A(String) }', 'String'),
                            self.key('pub enum Error { B(String) }', 'String'))

    def test_generic_impl_methods_retain_their_owner(self):
        source = 'impl<T, E> Request<T, E> { pub fn new() -> Self { todo!() } }'
        self.assertIn('Request', self.key(source, 'pub fn new()', 'an associated function'))

    def test_const_methods_use_the_method_name(self):
        source = 'impl Request { pub const fn with_sort() -> Self { todo!() } }'
        self.assertTrue(self.key(source, 'pub const fn with_sort()', 'a method').endswith('::with_sort'))

    def test_literal_braces_and_nested_comments_do_not_change_scopes(self):
        source = 'const X: &str = r#"{fake}"#; /* outer /* {nested} */ end */ pub struct Real { pub value: u64 }'
        self.assertEqual(self.key(source, 'pub value: u64'), 'struct Real::a struct field::value')


if __name__ == '__main__':
    unittest.main()
