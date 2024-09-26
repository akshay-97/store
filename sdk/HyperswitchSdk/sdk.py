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
        uri = "/create/" + intent.id + "/kaps"

        if self.client is not None:
            try:
                headers = {}
                if self.cell_id is not None:
                    headers = {"x-region": self.cell_id}

                response = self.client.get(
                    uri, headers=headers, name="IntentCreate")
                if response.status_code == 200:
                    response = response.json()
                    cell_id = parse_id(response["pi"])
                    self.payment_id = response["pi"]
                    self.cell_id = cell_id

                    return response
            except Exception as e:
                raise e
        else:
            raise Exception("Client is not set up yet")

    def create_payment_attempt(self, attempt: PaymentAttempt):
        if self.payment_id:
            uri = "/pay/" + \
                self.payment_id + "/" + attempt.attempt_id

            try:
                headers = {"x-region": self.cell_id}
                response = self.client.get(
                    uri, headers=headers, name="AttemptCreate" + " :" + self.cell_id)
                if response.status_code == 200:
                    response = response.json()
                    self.cell_id = parse_id(response["pa"])

                return response
            except Exception as e:
                raise e

    def update_attempt(self, attempt: PaymentAttempt):
        if self.payment_id is not None and self.cell_id is not None:
            uri = "/update_attempt/pay/" + attempt.attempt_id + "/" + self.payment_id

            headers = {"x-region": self.cell_id}

            try:
                response = self.client.get(
                    uri, headers=headers, name="UpdateAttempt" + " :" + self.cell_id)

                if response.status_code == 200:
                    return response.json()
            except Exception as e:
                raise e

    def update_intent(self, intent: PaymentIntent):
        if self.cell_id:
            uri = "/update_intent/" + intent.id

            headers = {"x-region": self.cell_id}

            try:
                response = self.client.get(
                    uri, headers=headers, name="UpdateIntent" + " :" + self.cell_id)
                if response.status_code == 200:
                    return response.json()
            except Exception as e:
                raise e

    def retrieve_intent(self, payment_id: str):
        if self.cell_id:
            uri = "/retrieve/payment_intent/" + payment_id

            headers = {"x-region": self.cell_id}
            try:
                response = self.client.get(
                    uri, headers=headers, name="RetrieveIntent" + " :" + self.cell_id)
                if response.status_code == 200:
                    return response.json()
            except Exception as e:
                raise e
