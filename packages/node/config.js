'use strict'

// `import 'gitvault/config'` or `require('gitvault/config')`: load the environment named by
// GITVAULT_ENV (or NODE_ENV) before anything else runs.
require('./index.js').config()
