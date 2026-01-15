/**
 * Multi-Tool Pattern
 * ==================
 *
 * This pattern shows how to:
 * 1. Organize multiple related tools
 * 2. Share code between tools
 * 3. Implement CRUD operations
 * 4. Use consistent response formats
 */

// Simulated data store
const store = new Map();

export function getMetadata() {
  return {
    name: "multi-tool-example",
    version: "1.0.0",
    description: "Demonstrates multi-tool skill patterns (CRUD)",
    author: "Skill Engine"
  };
}

/**
 * PATTERN: Group related tools together
 */
export function getTools() {
  return [
    // CRUD operations
    {
      name: "list",
      description: "List all items",
      parameters: [
        {
          name: "limit",
          paramType: "integer",
          description: "Maximum items to return (default: 10)",
          required: false
        }
      ]
    },
    {
      name: "get",
      description: "Get a single item by ID",
      parameters: [
        {
          name: "id",
          paramType: "string",
          description: "Item ID",
          required: true
        }
      ]
    },
    {
      name: "create",
      description: "Create a new item",
      parameters: [
        {
          name: "name",
          paramType: "string",
          description: "Item name",
          required: true
        },
        {
          name: "data",
          paramType: "string",
          description: "Item data (JSON)",
          required: false
        }
      ]
    },
    {
      name: "update",
      description: "Update an existing item",
      parameters: [
        {
          name: "id",
          paramType: "string",
          description: "Item ID",
          required: true
        },
        {
          name: "name",
          paramType: "string",
          description: "New name",
          required: false
        },
        {
          name: "data",
          paramType: "string",
          description: "New data (JSON)",
          required: false
        }
      ]
    },
    {
      name: "delete",
      description: "Delete an item",
      parameters: [
        {
          name: "id",
          paramType: "string",
          description: "Item ID",
          required: true
        }
      ]
    }
  ];
}

/**
 * PATTERN: Route to handlers using object dispatch
 */
export function executeTool(name, argsJson) {
  const args = JSON.parse(argsJson);

  // PATTERN: Handler map for clean routing
  const handlers = {
    'list': handleList,
    'get': handleGet,
    'create': handleCreate,
    'update': handleUpdate,
    'delete': handleDelete
  };

  const handler = handlers[name];
  if (!handler) {
    return {
      success: false,
      output: "",
      errorMessage: `Unknown tool: ${name}`
    };
  }

  // PATTERN: All handlers return same format
  return handler(args);
}

/**
 * PATTERN: List with pagination
 */
function handleList(args) {
  const { limit = 10 } = args;
  const items = Array.from(store.values()).slice(0, limit);

  return formatResponse({
    count: items.length,
    total: store.size,
    items: items
  });
}

/**
 * PATTERN: Get single item with not-found handling
 */
function handleGet(args) {
  const { id } = args;

  if (!id) {
    return formatError("ID is required");
  }

  const item = store.get(id);
  if (!item) {
    return formatError(`Item not found: ${id}`);
  }

  return formatResponse(item);
}

/**
 * PATTERN: Create with ID generation
 */
function handleCreate(args) {
  const { name, data } = args;

  if (!name) {
    return formatError("Name is required");
  }

  // Generate ID
  const id = `item-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

  // Parse optional JSON data
  let parsedData = {};
  if (data) {
    try {
      parsedData = JSON.parse(data);
    } catch (e) {
      return formatError("Invalid JSON in data parameter");
    }
  }

  const item = {
    id,
    name,
    data: parsedData,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString()
  };

  store.set(id, item);

  return formatResponse({
    message: "Item created",
    item
  });
}

/**
 * PATTERN: Update with partial updates
 */
function handleUpdate(args) {
  const { id, name, data } = args;

  if (!id) {
    return formatError("ID is required");
  }

  const item = store.get(id);
  if (!item) {
    return formatError(`Item not found: ${id}`);
  }

  // PATTERN: Only update provided fields
  if (name !== undefined) {
    item.name = name;
  }

  if (data !== undefined) {
    try {
      item.data = JSON.parse(data);
    } catch (e) {
      return formatError("Invalid JSON in data parameter");
    }
  }

  item.updatedAt = new Date().toISOString();
  store.set(id, item);

  return formatResponse({
    message: "Item updated",
    item
  });
}

/**
 * PATTERN: Delete with confirmation
 */
function handleDelete(args) {
  const { id } = args;

  if (!id) {
    return formatError("ID is required");
  }

  if (!store.has(id)) {
    return formatError(`Item not found: ${id}`);
  }

  store.delete(id);

  return formatResponse({
    message: `Item deleted: ${id}`
  });
}

/**
 * PATTERN: Consistent response format helpers
 */
function formatResponse(data) {
  return {
    success: true,
    output: JSON.stringify(data, null, 2),
    errorMessage: null
  };
}

function formatError(message) {
  return {
    success: false,
    output: "",
    errorMessage: message
  };
}

export function validateConfig(configJson) {
  return { ok: null };
}
