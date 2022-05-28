const toLocaleString = Date.prototype.toLocaleString;
const toLocaleDateString = Date.prototype.toLocaleDateString;
const toLocaleTimeString = Date.prototype.toLocaleTimeString;
Date.prototype.toLocaleString = function (locale?, options?) {
  return toLocaleString.call(this, locale || "en-US", {
    timeZone: "UTC",
    ...(options ?? {}),
  });
};
Date.prototype.toLocaleTimeString = function (locale?, options?) {
  return toLocaleTimeString.call(this, locale || "en-US", {
    timeZone: "UTC",
    ...(options ?? {}),
  });
};
Date.prototype.toLocaleDateString = function (locale?, options?) {
  return toLocaleDateString.call(this, locale || "en-US", {
    timeZone: "UTC",
    ...(options ?? {}),
  });
};
