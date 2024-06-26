terraform {
  required_providers {
    aws = {
      source = "hashicorp/aws"
      version = "~> 5.55"
    }
  }
}

provider "aws" {
  region = "ap-northeast-1" # 東京リージョン
}

variable "aws_key_name" {
  description = "The name of the AWS key pair to be used with the EC2 instance."
  type        = string
}

resource "aws_instance" "minecraft_server" {
  ami           = "ami-061a125c7c02edb39" # Amazon Linux 2023
  instance_type = "t3.large"
  key_name      = var.aws_key_name

  vpc_security_group_ids = [
    aws_security_group.allow_ssh.id,
    aws_security_group.allow_minecraft_port.id
  ]

  user_data = "${file("user_data.sh")}"

  tags = {
    Name = "minecraft-server"
  }
}

resource "aws_security_group" "allow_ssh" {
  name        = "allow_ssh"
  description = "Allow SSH inbound traffic"

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "allow_minecraft_port" {
  name        = "allow_minecraft_port"
  description = "Allow minecraft port inbound traffic"

  ingress {
    from_port   = 25565
    to_port     = 25565
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

output "instance_ip" {
  value = aws_instance.minecraft_server.public_ip
}
