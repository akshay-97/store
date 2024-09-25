import requests


class PaymentIntent:
    def __init__(self, id: str):
        self.id = id


class PaymentAttempt:
    def __init__(self, attempt_id: str):
        self.attempt_id = attempt_id


def parse_id(id: str):
    cell = id.split("_", 1)
    return cell[0]


class HyperswitchSdk:
    client = None
    base_url = ""

    def __init__(self, client):
        self.client = client
        self.payment_id = None
        self.cell_id = None

    def create_payment_intent(self, intent: PaymentIntent):
        uri = "http://3.235.16.150:8000/create/" + intent.id + "/kaps"

        if self.client is not None:
            try:
                headers = {}
                if self.cell_id is not None:
                    headers = {"x-region": self.cell_id}

                response = self.client.get(uri, headers=headers).json()
                cell_id = parse_id(response["pi"])
                self.payment_id = response["pi"]
                return response
            except Exception as e:
                raise e
            finally:
                if self.cell_id is None:
                    self.cell_id = cell_id

        else:
            raise Exception("Client is not set up yet")

    def create_payment_attempt(self, attempt: PaymentAttempt):
        uri = "http://3.235.16.150:8000/pay/" + \
            self.payment_id + "/" + attempt.attempt_id

        if self.cell_id is None:
            raise Exception("Cannot make this request without cell id")

        headers = {"x-region": self.cell_id}

        try:
            return self.client.get(uri, headers=headers).json()
        except Exception as e:
            raise e

    def update_attempt(self, attempt: PaymentAttempt):
        uri = "/update_attempt/pay/" + attempt.attempt_id + "/" + self.payment_id

        if self.cell_id is None:
            raise Exception("This cannot be called without cell_id")

        headers = {"x-region": self.cell_id}

        try:
            return self.client.get(uri, headers=headers).json()
        except Exception as e:
            raise e

    def update_intent(self, intent: PaymentIntent):
        uri = "/update_intent/" + intent.id

        if self.cell_id is None:
            raise Exception("This cannot be called without cell_id")

        headers = {"x-region": self.cell_id}

        try:
            return self.client.get(uri, headers=headers).json()
        except Exception as e:
            raise e

    def retrieve_intent(self, payment_id: str):
        uri = "/retrieve/payment_intent/" + payment_id

        if self.cell_id is None:
            raise Exception("This cannot be called without cell_id")

        headers = {"x-region": self.cell_id}
        try:
            return self.client.get(uri, headers=headers).json()
        except Exception as e:
            raise e


session = HyperswitchSdk(requests.Session())
session.create_payment_intent(PaymentIntent("gagagagaaglagjkag"))
session.create_payment_attempt(PaymentAttempt("gagagagaaglagjkag_v1_v1"))
