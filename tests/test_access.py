"""Tests for the project's access-control implementation."""

import importlib
import inspect

import pytest


def _access_callable():
	"""Find the conventional access-check function used by the project."""
	for module_name in ("access", "src.access", "app.access"):
		try:
			module = importlib.import_module(module_name)
		except ModuleNotFoundError:
			continue

		for name in ("has_access", "can_access", "is_allowed", "check_access"):
			candidate = getattr(module, name, None)
			if callable(candidate):
				return candidate

	pytest.skip("No access-check implementation was found")


def _check_access(check, user, resource, permission):
	"""Call an access checker using its declared argument names."""
	parameters = inspect.signature(check).parameters
	values = {
		"user": user,
		"resource": resource,
		"permission": permission,
		"action": permission,
	}
	kwargs = {name: values[name] for name in parameters if name in values}
	if len(kwargs) == len(parameters):
		return check(**kwargs)
	return check(user, resource, permission)


@pytest.mark.parametrize("permission", ["read", "write", "delete"])
def test_authenticated_user_with_permission_is_allowed(permission):
	check = _access_callable()
	user = {"authenticated": True, "permissions": [permission]}

	assert _check_access(check, user, "document", permission) is True


def test_unauthenticated_user_is_denied():
	check = _access_callable()
	user = {"authenticated": False, "permissions": ["read", "write"]}

	assert _check_access(check, user, "document", "read") is False


def test_user_without_required_permission_is_denied():
	check = _access_callable()
	user = {"authenticated": True, "permissions": ["read"]}

	assert _check_access(check, user, "document", "delete") is False
