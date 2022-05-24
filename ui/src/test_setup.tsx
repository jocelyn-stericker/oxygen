const toLocaleString = Date.prototype.toLocaleString;
const toLocaleDateString = Date.prototype.toLocaleDateString;
const toLocaleTimeString = Date.prototype.toLocaleTimeString;
Date.prototype.toLocaleString = function (locale = "en-US", options?) {
  return toLocaleString.call(this, "en-US", {
    timeZone: "UTC",
    ...(options ?? {}),
  });
};
Date.prototype.toLocaleTimeString = function (locale = "en-US", options?) {
  return toLocaleTimeString.call(this, "en-US", {
    timeZone: "UTC",
    ...(options ?? {}),
  });
};
Date.prototype.toLocaleDateString = function (locale = "en-US", options?) {
  return toLocaleDateString.call(this, "en-US", {
    timeZone: "UTC",
    ...(options ?? {}),
  });
};
