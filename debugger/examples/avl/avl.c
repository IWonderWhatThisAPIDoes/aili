/**
 * Showcases repeated insertion into an
 * [AVL tree](https://en.wikipedia.org/wiki/AVL_tree).
 * 
 * Demonstrates the use of Aili for rendering tree structures,
 * including with ordered children.
 * 
 * @file
 */

#include <stdlib.h>
#include <assert.h>

typedef struct tree_node {
    struct tree_node* left, * right, * parent;
    int imbalance;
    int key;
} tree_node;

typedef struct tree {
    tree_node* root;
} tree;

// Get pointer to parent node's pointer to this node
#define TREE_PARENTPTR(ptree, pnode) \
    ((pnode)->parent ? \
    (pnode) == (pnode)->parent->left ? \
    &(pnode)->parent->left : \
    &(pnode)->parent->right : \
    &(ptree)->root)
// Check relative position of a node and its parent
// -1 is left, 1 is right. 0 for root node
#define TREE_SIDEOF(pnode) ((pnode)->parent ? (pnode) == (pnode)->parent->left ? -1 : 1 : 0)

/**
 * Creates an empty tree
 * 
 * @remark If the tree is filled in, caller must call tree_delete
 * @remark Value objects must align with tree_node_t        
 */
tree tree_init() {
    tree t = { NULL };
    return t;
}

/**
 * Destroys a tree and frees its memory
 * 
 * @param[inout] t The tree
 * 
 * @remark Destructors are called on all contained keys and values
 */
void tree_delete(tree* t) {
    if (t->root) {
        tree_delete_recursive(t, t->root);
    }
}

/**
 * Destroys a subtree recursively.
 * Helper function for tree_delete
 * 
 * @param[inout] t    The tree
 * @param[inout] root Root of the subtree to delete
 */
void tree_delete_recursive(tree* t, tree_node* root)
{
    // Destroy subtrees first
    if (root->left) tree_delete_recursive(t, root->left);
    if (root->right) tree_delete_recursive(t, root->right);

    free(root);
}

/**
 * Inserts a node into a tree
 * 
 * @param[inout] t The tree
 * @param[in]  key The key to add
 * 
 * @remark If key is already present, this is equivalent to tree_find
 */
void tree_put(tree* t, int key) {
    tree_node** dest = &t->root, * parent = NULL;

    while (*dest)
    {
        parent = *dest;

        if      (key > parent->key) dest = &parent->right;
        else if (key < parent->key) dest = &parent->left;
        else return;
    }

    // Create the new node
    tree_node* newNode = NULL;
    if (!parent) newNode = tree_create_root(t);
    else         newNode = tree_insert_under(t, parent, dest == &parent->right);
    if (!newNode) return;

    // Initialize key
    newNode->key = key;
}

/**
 * Creates a new node and places it under an existing one
 * 
 * @param[inout]      t The tree that the nodes belong to
 * @param[inout] parent Parent node
 * @param[in]     right Nonzero to place new node to the right, zero otherwise
 * @return              The new node, or NULL if memory allocation fails
 * 
 * @remark Emptiness of the leaf is not checked.
 *         If parent has a child on the specified side, it gets overwritten and leaks
 * @remark The new node may be rotated away (so it will not be at parent->left or parent->right),
 *         so the returned address should be relied on instead
 */
tree_node* tree_insert_under(tree* t, tree_node* parent, int right) {
    tree_node* node = (tree_node*)malloc(sizeof(tree_node));
    if (!node) return NULL;

    // Initialize
    tree_node init = { NULL, NULL, parent, 0, 0 };
    *node = init;

    // Set the pointer on parent
    if (right) parent->right = node;
    else parent->left = node;

    // Rebalance
    tree_rebalance(t, parent, right ? 1 : -1);

    return node;
}

/**
 * Creates a new node and places it to the root position
 * 
 * @param[inout] t The tree
 * @return         The new node, or NULL if memory allocation fails
 * 
 * @remark Emptiness of the tree is not checked.
 *          If the tree has a root, it gets overwritten and leaks
 */
tree_node* tree_create_root(tree* t) {
    t->root = (tree_node*)malloc(sizeof(tree_node));
    if (!t->root) return NULL;

    tree_node init = { NULL, NULL, NULL, 0, 0 };
    *t->root = init;

    return t->root;
}

/**
 * Updates imbalance of a tree node.
 * Imbalance is updated recursively upwards
 * 
 * @param[inout]    t The tree that the node belongs to
 * @param[inout] node The node
 * @param[in]      bf Local change of balance factor, should be 1 or -1
 * 
 */
void tree_rebalance(tree* t, tree_node* node, int bf) {
    assert(node->imbalance == 1 || node->imbalance == 0 || node->imbalance == -1);
    assert(bf == 1 || bf == -1);
    node->imbalance += bf;

    // Therefore, valid values are now...
    assert(node->imbalance == 0 || node->imbalance == bf || node->imbalance == 2 * bf);

    switch (node->imbalance * bf)
    {
    case 0:
        // We added a node, so depth of one branch now matches
        // depth of the other. Total subtree depth remains unchanged.
        break;

    case 1:
        // We added a node to a balanced tree, so depth of one
        // branch has increased, and so has the depth of the whole tree.
        // Propagate the change upwards.
        if (node->parent) tree_rebalance(t, node->parent, TREE_SIDEOF(node));
        break;

    case 2:
        // We added a node to an already imbalanced branch.
        // Now we need to rotate.
        tree_rotate(TREE_PARENTPTR(t, node));
        // Rotation reduces total subtree depth by 1, which is
        // the one we have just added by inserting this node.
        // Total subtree depth remains unchanged. So does parent imbalance.
        break;
    }

    assert(node->imbalance == 1 || node->imbalance == 0 || node->imbalance == -1);
}

/**
 * Rotates tree around a node if necessary
 * 
 * @param[inout] node The node pointer to rotate around
 * 
 * @details Performs an AVL tree rotation
 * 
 *     * <-- *node (the node pointer)
 *     |
 *     o <-- **node (the node itself)
 *    / \
 *   o   o
 *
 *     * <-- *node (modified to point to different node)
 *     |
 *     o <-- Formerly *(*node)->left
 *      \
 *       o <-- Formerly **node
 *        \
 *         o <-- Formerly *(*node)->right
 * 
 *    (this is only one of possible rotations)
 */
void tree_rotate(tree_node** node) {
    const int imbalance = (*node)->imbalance;

    if (imbalance == 2)
    {
        if ((*node)->right->imbalance < 0)
            tree_right_rotate(&(*node)->right);
        tree_left_rotate(node);
    }
    else if (imbalance == -2)
    {
        if ((*node)->left->imbalance > 0)
            tree_left_rotate(&(*node)->left);
        tree_right_rotate(node);
    }
}

/**
 * Left-rotates around a node
 * 
 * @param[inout] node The node pointer to rotate around
 */
void tree_left_rotate(tree_node** node) {
    /*
     *   p               p
     *   |               |
     *  (a)             (b)
     *  / \             / \
     * d  (b)   -->   (a)  e
     *    / \         / \
     *   c   e       d   c
     */
    tree_node* a = *node,
        * b = a->right,
        * c = b->left,
        * p = a->parent;

    assert(a->imbalance > 0);

    *node = b; // *node is p->left or p->right
    b->parent = p;
    a->parent = b;
    b->left = a;
    if (c) c->parent = a;
    a->right = c;

    // a->imbalance (old) = max(depth(c), depth(e)) + 1 - depth(d)
    // b->imbalance (old) = depth(e) - depth(c)
    // a->imbalance (new) = depth(c) - depth(d)
    // b->imbalance (new) = depth(e) - 1 - max(depth(d), depth(c))
    a->imbalance += -1 - (b->imbalance > 0 ? b->imbalance : 0);
    b->imbalance += -1 + (a->imbalance > 0 ? a->imbalance : 0);

    assert(a->imbalance >= -1 && a->imbalance <= 1);
    assert(b->imbalance >= -1 && b->imbalance <= 1);
}

/**
 * Right-rotates around a node
 * 
 * @param[inout] node The node pointer to rotate around
 */
void tree_right_rotate(tree_node** node) {
    assert((*node)->imbalance < 0);
    /*
     *     p               p
     *     |               |
     *    (a)             (b)
     *    / \             / \
     *  (b)  d    -->    e  (a)
     *  / \                 / \
     * e   c               c   d
     */
    tree_node* a = *node,
        * b = a->left,
        * c = b->right,
        * p = a->parent;

    *node = b; // *node is p->left or p->right
    b->parent = p;
    a->parent = b;
    b->right = a;
    if (c) c->parent = a;
    a->left = c;

    a->imbalance += 1 - (b->imbalance < 0 ? b->imbalance : 0);
    b->imbalance += 1 + (a->imbalance < 0 ? a->imbalance : 0);

    assert(a->imbalance >= -1 && a->imbalance <= 1);
    assert(b->imbalance >= -1 && b->imbalance <= 1);
}

int main(void) {
    tree t = tree_init();
    tree_put(&t, 12);
    tree_put(&t, 4);
    tree_put(&t, 5);
    tree_put(&t, 6);
    tree_put(&t, 2);
    tree_put(&t, 8);
    tree_put(&t, 10);
    tree_put(&t, 2);
    tree_put(&t, 1);
    tree_delete(&t);
}
