import test from 'ava';

import { getSelectionText } from '../index.js';

test('Invoke getSelectionText', (t) => {
	getSelectionText();
	t.pass();
});
