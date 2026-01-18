{ pkgs, llm-assistant }:
pkgs.dockerTools.buildLayeredImage {
  name = "ghcr.io/ofalvai/homecooked-llm-assistant";
  tag = "latest";
  config = {
    WorkingDir = "/app";
    Cmd = [
      "${llm-assistant}/bin/llm-assistant"
      "--config"
      "/data/config/config.ini"
      "server"
    ];
    ExposedPorts = {
      "8080/tcp" = { };
    };
    Volumes = {
      "/data/config" = { };
    };
    Env = [
      "CONFIG=/data/config/config.ini"
      "PORT=8080"
    ];
  };
}
