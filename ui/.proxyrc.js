module.exports = function (app) {
  app.use(function (req, res, next) {
    // Prevent browser caching of all URLs (except /api and /ws because we proxy those)
    if (
      !req.path ||
      (!req.path.startsWith("/api") && !req.path.startsWith("/ws"))
    ) {
      res.setHeader("Cache-Control", "no-cache, no-store, must-revalidate");
    }
    next();
  });
};
