# Tombi Extension Toolkit

## Overview

This crate provides implementation-independent APIs, shared result types, and
helpers for implementing Tombi extensions.

## Public API

- `ast`: operations over TOML syntax without exposing its storage
- `document_tree`: operations over semantic TOML values without exposing their storage
- Shared types and helpers for completion, code actions, document links, hover,
  and inlay hints
- Cache, URI, and text-edit helpers used by extension implementations

The `ast` and `document_tree` modules re-export interface crates. They do not
expose Tombi's source-backed syntax or document-tree implementations.

## Integration

Extension implementations compose these APIs directly. Registration, lifecycle,
and dispatch are owned by the host application; this crate intentionally does
not define an empty marker trait or an inheritance hierarchy.
